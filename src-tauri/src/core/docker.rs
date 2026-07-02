use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct DockerContainerStats {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_usage: String,
    pub net_io: String,
    pub block_io: String,
    pub pids: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: String,
    pub created_at: String,
    pub size: String,
    pub stats: Option<DockerContainerStats>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created_since: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerVolume {
    pub driver: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerComposeProject {
    pub name: String,
    pub status: String,
    pub config_files: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerComposeServiceContainer {
    pub id: String,
    pub name: String,
    pub state: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DockerComposeService {
    pub name: String,
    pub status: String,
    pub containers: Vec<DockerComposeServiceContainer>,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct RemoteDockerOverview {
    pub available: bool,
    pub version: String,
    pub compose_available: bool,
    pub containers: Vec<DockerContainer>,
    pub images: Vec<DockerImage>,
    pub volumes: Vec<DockerVolume>,
    pub networks: Vec<DockerNetwork>,
    pub compose_projects: Vec<DockerComposeProject>,
}

pub const DOCKER_OVERVIEW_SCRIPT: &str = r#"sh -c '
if ! command -v docker >/dev/null 2>&1; then
  printf "DOCKER_AVAILABLE\t0\n"
  exit 0
fi

if ! docker info >/dev/null 2>&1; then
  printf "DOCKER_AVAILABLE\t0\n"
  docker info 2>&1 | head -n 4 >&2
  exit 0
fi

printf "DOCKER_AVAILABLE\t1\n"
version=$(docker version --format "{{.Server.Version}}" 2>/dev/null || true)
version=$(printf "%s" "$version" | tr "\t\r\n" "   ")
printf "DOCKER_VERSION\t%s\n" "$version"

docker ps -a --no-trunc --format "CONTAINER\t{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.State}}\t{{.Ports}}\t{{.CreatedAt}}\t{{.Size}}" 2>/dev/null
docker stats --no-stream --no-trunc --format "CONTAINER_STATS\t{{.ID}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}\t{{.PIDs}}" 2>/dev/null
docker images --no-trunc --format "IMAGE\t{{.ID}}\t{{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}" 2>/dev/null
docker volume ls --format "VOLUME\t{{.Driver}}\t{{.Name}}" 2>/dev/null
docker network ls --no-trunc --format "NETWORK\t{{.ID}}\t{{.Name}}\t{{.Driver}}\t{{.Scope}}" 2>/dev/null

if docker compose version >/dev/null 2>&1; then
  printf "COMPOSE_AVAILABLE\t1\n"
  printf "COMPOSE_JSON_BEGIN\n"
  docker compose ls --format json 2>/dev/null || true
  printf "\nCOMPOSE_JSON_END\n"
else
  printf "COMPOSE_AVAILABLE\t0\n"
fi
'"#;

pub fn parse_docker_overview_output(output: &str) -> RemoteDockerOverview {
    let mut overview = RemoteDockerOverview::default();
    let mut stats_rows: Vec<(String, DockerContainerStats)> = Vec::new();
    let mut compose_json = String::new();
    let mut in_compose_json = false;

    for line in output.lines() {
        if line == "COMPOSE_JSON_BEGIN" {
            in_compose_json = true;
            continue;
        }
        if line == "COMPOSE_JSON_END" {
            in_compose_json = false;
            continue;
        }
        if in_compose_json {
            compose_json.push_str(line);
            compose_json.push('\n');
            continue;
        }

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.is_empty() {
            continue;
        }

        match cols[0] {
            "DOCKER_AVAILABLE" if cols.len() >= 2 => {
                overview.available = cols[1] == "1";
            }
            "DOCKER_VERSION" if cols.len() >= 2 => {
                overview.version = cols[1].to_string();
            }
            "COMPOSE_AVAILABLE" if cols.len() >= 2 => {
                overview.compose_available = cols[1] == "1";
            }
            "CONTAINER" if cols.len() >= 9 => overview.containers.push(DockerContainer {
                id: cols[1].to_string(),
                name: cols[2].to_string(),
                image: cols[3].to_string(),
                status: cols[4].to_string(),
                state: cols[5].to_string(),
                ports: cols[6].to_string(),
                created_at: cols[7].to_string(),
                size: cols[8].to_string(),
                stats: None,
            }),
            "CONTAINER_STATS" if cols.len() >= 8 => {
                stats_rows.push((
                    cols[1].to_string(),
                    DockerContainerStats {
                        cpu_percent: parse_percent(cols[2]),
                        memory_usage: cols[3].to_string(),
                        memory_percent: parse_percent(cols[4]),
                        net_io: cols[5].to_string(),
                        block_io: cols[6].to_string(),
                        pids: cols[7].to_string(),
                    },
                ));
            }
            "IMAGE" if cols.len() >= 6 => overview.images.push(DockerImage {
                id: cols[1].to_string(),
                repository: cols[2].to_string(),
                tag: cols[3].to_string(),
                size: cols[4].to_string(),
                created_since: cols[5].to_string(),
            }),
            "VOLUME" if cols.len() >= 3 => overview.volumes.push(DockerVolume {
                driver: cols[1].to_string(),
                name: cols[2].to_string(),
            }),
            "NETWORK" if cols.len() >= 5 => overview.networks.push(DockerNetwork {
                id: cols[1].to_string(),
                name: cols[2].to_string(),
                driver: cols[3].to_string(),
                scope: cols[4].to_string(),
            }),
            _ => {}
        }
    }

    for container in &mut overview.containers {
        if let Some((_, stats)) = stats_rows.iter().find(|(id, _)| {
            id == &container.id || container.id.starts_with(id) || id.starts_with(&container.id)
        }) {
            container.stats = Some(stats.clone());
        }
    }

    overview.compose_projects = parse_compose_projects(&compose_json);
    overview
}

fn parse_percent(value: &str) -> f64 {
    value.trim().trim_end_matches('%').parse().unwrap_or(0.0)
}

fn parse_compose_projects(raw: &str) -> Vec<DockerComposeProject> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            let name = item
                .get("Name")
                .or_else(|| item.get("name"))
                .and_then(serde_json::Value::as_str)?;
            let status = item
                .get("Status")
                .or_else(|| item.get("status"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let config_files = item
                .get("ConfigFiles")
                .or_else(|| item.get("configFiles"))
                .or_else(|| item.get("ConfigFiles"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");

            Some(DockerComposeProject {
                name: name.to_string(),
                status: status.to_string(),
                config_files: config_files.to_string(),
            })
        })
        .collect()
}

pub fn parse_compose_services_output(
    services_raw: &str,
    ps_json_raw: &str,
) -> Vec<DockerComposeService> {
    let mut services: Vec<DockerComposeService> = services_raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|name| DockerComposeService {
            name: name.to_string(),
            status: String::new(),
            containers: Vec::new(),
        })
        .collect();

    for item in parse_compose_ps_json_values(ps_json_raw) {
        let Some(service_name) = json_string_field(&item, &["Service", "service"]) else {
            continue;
        };

        if !services.iter().any(|service| service.name == service_name) {
            services.push(DockerComposeService {
                name: service_name.to_string(),
                status: String::new(),
                containers: Vec::new(),
            });
        }

        let container = DockerComposeServiceContainer {
            id: json_string_field(&item, &["ID", "Id", "id"])
                .unwrap_or("")
                .to_string(),
            name: json_string_field(&item, &["Name", "name"])
                .unwrap_or("")
                .to_string(),
            state: json_string_field(&item, &["State", "state"])
                .unwrap_or("")
                .to_string(),
            status: json_string_field(&item, &["Status", "status"])
                .unwrap_or("")
                .to_string(),
        };

        if let Some(service) = services
            .iter_mut()
            .find(|service| service.name == service_name)
        {
            service.containers.push(container);
        }
    }

    for service in &mut services {
        service
            .containers
            .sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        service.status = compose_service_status(&service.containers);
    }

    services
}

fn parse_compose_ps_json_values(raw: &str) -> Vec<serde_json::Value> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(items) = value.as_array() {
            return items.clone();
        }
        if value.is_object() {
            return vec![value];
        }
    }

    raw.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line.trim()).ok())
        .filter(|value| value.is_object())
        .collect()
}

fn json_string_field<'a>(item: &'a serde_json::Value, names: &[&str]) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| item.get(*name).and_then(serde_json::Value::as_str))
}

fn compose_service_status(containers: &[DockerComposeServiceContainer]) -> String {
    if containers.is_empty() {
        return String::new();
    }

    if containers
        .iter()
        .any(|container| container.state.eq_ignore_ascii_case("running"))
    {
        return "running".to_string();
    }

    let mut states: Vec<&str> = Vec::new();
    for container in containers {
        let state = container.state.trim();
        if !state.is_empty()
            && !states
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(state))
        {
            states.push(state);
        }
    }

    if states.is_empty() {
        containers
            .iter()
            .find_map(|container| {
                let status = container.status.trim();
                (!status.is_empty()).then_some(status)
            })
            .unwrap_or("")
            .to_string()
    } else {
        states.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_overview_rows() {
        let raw = concat!(
            "DOCKER_AVAILABLE\t1\n",
            "DOCKER_VERSION\t26.1.0\n",
            "CONTAINER\tabc123\tweb\tnginx:latest\tUp 2 minutes\trunning\t0.0.0.0:80->80/tcp\t2026-01-01 00:00:00 +0000 UTC\t0B\n",
            "CONTAINER_STATS\tabc123\t1.25%\t10MiB / 1GiB\t0.98%\t1kB / 2kB\t0B / 0B\t3\n",
            "IMAGE\tsha256:fff\tnginx\tlatest\t70MB\t2 days ago\n",
            "VOLUME\tlocal\tdata\n",
            "NETWORK\tdef456\tbridge\tbridge\tlocal\n",
            "COMPOSE_AVAILABLE\t1\n",
            "COMPOSE_JSON_BEGIN\n",
            "[{\"Name\":\"demo\",\"Status\":\"running(1)\",\"ConfigFiles\":\"/srv/demo/compose.yaml\"}]\n",
            "COMPOSE_JSON_END\n",
        );

        let overview = parse_docker_overview_output(raw);

        assert!(overview.available);
        assert_eq!(overview.version, "26.1.0");
        assert_eq!(
            overview.containers[0].stats.as_ref().unwrap().cpu_percent,
            1.25
        );
        assert_eq!(overview.images[0].repository, "nginx");
        assert_eq!(overview.volumes[0].name, "data");
        assert_eq!(overview.networks[0].name, "bridge");
        assert_eq!(overview.compose_projects[0].name, "demo");
    }

    #[test]
    fn parses_missing_docker_state() {
        let overview = parse_docker_overview_output("DOCKER_AVAILABLE\t0\n");
        assert!(!overview.available);
        assert!(overview.containers.is_empty());
    }

    #[test]
    fn parses_compose_service_json_array_output() {
        let services = parse_compose_services_output(
            "web\nworker\n",
            r#"[{"Service":"web","Name":"demo-web-1","ID":"abc","State":"running","Status":"Up 2 minutes"}]"#,
        );

        assert_eq!(services.len(), 2);
        assert_eq!(services[0].name, "web");
        assert_eq!(services[0].status, "running");
        assert_eq!(services[0].containers[0].name, "demo-web-1");
        assert_eq!(services[1].name, "worker");
        assert!(services[1].containers.is_empty());
    }

    #[test]
    fn parses_compose_service_newline_json_output() {
        let raw = concat!(
            "{\"Service\":\"web\",\"Name\":\"demo-web-1\",\"ID\":\"abc\",\"State\":\"running\",\"Status\":\"Up\"}\n",
            "{\"Service\":\"web\",\"Name\":\"demo-web-2\",\"ID\":\"def\",\"State\":\"exited\",\"Status\":\"Exited\"}\n",
        );
        let services = parse_compose_services_output("web\n", raw);

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].containers.len(), 2);
        assert_eq!(services[0].containers[0].id, "abc");
    }

    #[test]
    fn includes_services_declared_without_created_containers() {
        let services = parse_compose_services_output("db\ncache\n", "");

        assert_eq!(services.len(), 2);
        assert_eq!(services[0].name, "db");
        assert!(services[0].status.is_empty());
        assert!(services[0].containers.is_empty());
    }
}
