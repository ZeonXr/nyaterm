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

if docker compose version >/dev/null 2>&1; then
  printf "COMPOSE_AVAILABLE\t1\n"
else
  printf "COMPOSE_AVAILABLE\t0\n"
fi
'"#;

pub const DOCKER_IMAGES_SCRIPT: &str = r#"docker images --no-trunc --format "IMAGE\t{{.ID}}\t{{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}" 2>/dev/null"#;

pub const DOCKER_VOLUMES_SCRIPT: &str =
    r#"docker volume ls --format "VOLUME\t{{.Driver}}\t{{.Name}}" 2>/dev/null"#;

pub const DOCKER_NETWORKS_SCRIPT: &str = r#"docker network ls --no-trunc --format "NETWORK\t{{.ID}}\t{{.Name}}\t{{.Driver}}\t{{.Scope}}" 2>/dev/null"#;

pub const DOCKER_COMPOSE_PROJECTS_SCRIPT: &str = r#"sh -c '
if ! docker compose version >/dev/null 2>&1; then
  exit 0
fi
docker compose ls --format json 2>/dev/null || true
'"#;

pub const DOCKER_CONTAINER_DETAILS_INSPECT_BEGIN: &str = "INSPECT_JSON_BEGIN";
pub const DOCKER_CONTAINER_DETAILS_INSPECT_END: &str = "INSPECT_JSON_END";
pub const DOCKER_CONTAINER_DETAILS_STATS_BEGIN: &str = "CONTAINER_STATS_BEGIN";
pub const DOCKER_CONTAINER_DETAILS_STATS_END: &str = "CONTAINER_STATS_END";

pub fn docker_container_details_script(container_id: &str) -> String {
    format!(
        "printf '{inspect_begin}\\n'; \
         docker inspect {container_id} || exit $?; \
         printf '\\n{inspect_end}\\n'; \
         printf '{stats_begin}\\n'; \
         docker stats --no-stream --no-trunc --format \"CONTAINER_STATS\\t{{{{.ID}}}}\\t{{{{.CPUPerc}}}}\\t{{{{.MemUsage}}}}\\t{{{{.MemPerc}}}}\\t{{{{.NetIO}}}}\\t{{{{.BlockIO}}}}\\t{{{{.PIDs}}}}\" {container_id} 2>/dev/null || true; \
         printf '\\n{stats_end}\\n'",
        container_id = sh_quote_local(container_id),
        inspect_begin = DOCKER_CONTAINER_DETAILS_INSPECT_BEGIN,
        inspect_end = DOCKER_CONTAINER_DETAILS_INSPECT_END,
        stats_begin = DOCKER_CONTAINER_DETAILS_STATS_BEGIN,
        stats_end = DOCKER_CONTAINER_DETAILS_STATS_END,
    )
}

fn sh_quote_local(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
