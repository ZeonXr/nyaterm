import { Check, Copy } from "lucide-react";
import type { ReactNode } from "react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { DockerContainer } from "@/types/global";

type DockerStateKind = "danger" | "running" | "stopped" | "transition" | "unknown";

interface DockerContainerDetailsDialogProps {
  container: DockerContainer | null;
  onOpenChange: (open: boolean) => void;
}

export default function DockerContainerDetailsDialog({
  container,
  onOpenChange,
}: DockerContainerDetailsDialogProps) {
  const { t } = useTranslation();
  const stats = container?.stats;
  const ports = parseDockerPorts(container?.ports ?? "");
  const portValue = ports.length > 0 ? ports.map(formatPort).join("\n") : "-";

  return (
    <Dialog open={Boolean(container)} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(620px,calc(100vw-2rem))] sm:max-w-[620px] p-0 gap-0 overflow-hidden">
        <DialogHeader className="border-b bg-muted/10 px-5 py-4 pr-12">
          <DialogTitle className="flex min-w-0 items-center gap-2 text-sm">
            <span className="min-w-0 truncate" title={container?.name}>
              {container?.name ?? t("dockerManager.containerDetails")}
            </span>
            {container ? <StateBadge state={container.state} /> : null}
          </DialogTitle>
          <DialogDescription className="sr-only">
            {t("dockerManager.containerDetails")}
          </DialogDescription>
        </DialogHeader>

        {container ? (
          <div className="max-h-[75vh] overflow-y-auto p-4 terminal-scroll">
            <div className="space-y-4">
              <ContainerSnapshot container={container} />

              <DetailSection title={t("dockerManager.identity")}>
                <CopyableDetailRow
                  label={t("dockerManager.containerName")}
                  value={container.name}
                />
                <CopyableDetailRow
                  label={t("dockerManager.containerId")}
                  value={container.id}
                  displayValue={middleEllipsis(container.id, 28, 12)}
                />
                <CopyableDetailRow label={t("dockerManager.image")} value={container.image} />
                <DetailRow
                  label={t("dockerManager.status")}
                  value={container.status || container.state}
                />
                <DetailRow
                  label={t("dockerManager.createdAt")}
                  value={container.created_at || "-"}
                />
                <DetailRow label={t("dockerManager.size")} value={container.size || "-"} />
              </DetailSection>

              <DetailSection title={t("dockerManager.networking")}>
                <CopyableDetailRow
                  label={t("dockerManager.ports")}
                  value={portValue}
                  displayValue={portValue}
                  multiline
                />
              </DetailSection>

              <DetailSection title={t("dockerManager.io")}>
                <DetailRow label={t("dockerManager.netIo")} value={stats?.net_io || "-"} />
                <DetailRow label={t("dockerManager.blockIo")} value={stats?.block_io || "-"} />
              </DetailSection>
            </div>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

function ContainerSnapshot({ container }: { container: DockerContainer }) {
  const { t } = useTranslation();
  const stats = container.stats;

  return (
    <div className="rounded-md border border-border/70 bg-muted/[0.04] p-3">
      <div className="mb-3 grid min-w-0 grid-cols-[minmax(0,1fr)_auto] gap-3">
        <div className="min-w-0">
          <div className="truncate font-mono text-xs text-foreground/90" title={container.image}>
            {container.image}
          </div>
          <div className="mt-1 truncate font-mono text-[0.6875rem] text-muted-foreground">
            {middleEllipsis(container.id, 22, 10)}
          </div>
        </div>
        <div className="shrink-0 font-mono text-[0.6875rem] text-muted-foreground">
          {container.size || "-"}
        </div>
      </div>
      <DetailGrid>
        <DetailMetric
          label={t("dockerManager.cpu")}
          tone={(stats?.cpu_percent ?? 0) > 0 ? "active" : undefined}
          value={`${stats?.cpu_percent.toFixed(1) ?? "0.0"}%`}
        />
        <DetailMetric
          label={t("dockerManager.memory")}
          value={stats ? formatMemoryUsed(stats.memory_usage) : "-"}
        />
        <DetailMetric label={t("dockerManager.pids")} value={stats?.pids || "-"} />
      </DetailGrid>
    </div>
  );
}

function DetailSection({ children, title }: { children: ReactNode; title: string }) {
  return (
    <section className="space-y-2">
      <h3 className="px-0.5 text-[0.6875rem] font-semibold uppercase tracking-wide text-muted-foreground">
        {title}
      </h3>
      <div className="divide-y rounded-md border border-border/70 bg-muted/[0.03]">{children}</div>
    </section>
  );
}

function CopyableDetailRow({
  displayValue,
  label,
  multiline,
  value,
}: {
  displayValue?: string;
  label: string;
  multiline?: boolean;
  value: string;
}) {
  const { t } = useTranslation();

  return (
    <div className="group/row grid min-w-0 grid-cols-[6rem_minmax(0,1fr)_1.5rem] items-start gap-2 px-3 py-2 text-xs transition-colors hover:bg-muted/20">
      <span className="truncate pt-1 text-muted-foreground">{label}</span>
      <span
        className={cn(
          "min-w-0 pt-1 font-mono text-foreground/90",
          multiline ? "whitespace-pre-wrap break-all" : "truncate",
        )}
        title={value}
      >
        {displayValue ?? value}
      </span>
      <CopyValueButton label={t("common.copyToClipboard")} value={value} />
    </div>
  );
}

function DetailRow({
  displayValue,
  label,
  multiline,
  value,
}: {
  displayValue?: string;
  label: string;
  multiline?: boolean;
  value: string;
}) {
  return (
    <div className="grid min-w-0 grid-cols-[6rem_minmax(0,1fr)] items-start gap-2 px-3 py-2 text-xs transition-colors hover:bg-muted/20">
      <span className="truncate pt-1 text-muted-foreground">{label}</span>
      <span
        className={cn(
          "min-w-0 pt-1 font-mono text-foreground/90",
          multiline ? "whitespace-pre-wrap break-all" : "truncate",
        )}
        title={value}
      >
        {displayValue ?? value}
      </span>
    </div>
  );
}

function DetailGrid({ children }: { children: ReactNode }) {
  return <div className="grid grid-cols-3 gap-2">{children}</div>;
}

function DetailMetric({
  label,
  tone,
  value,
}: {
  label: string;
  tone?: "active" | "hot";
  value: string;
}) {
  return (
    <div
      className={cn(
        "min-w-0 rounded-md bg-background/35 px-2.5 py-2",
        tone === "active" && "text-sky-300",
        tone === "hot" && "text-red-300",
      )}
    >
      <div className="truncate text-[0.625rem] text-muted-foreground">{label}</div>
      <div className="truncate font-mono text-xs text-foreground/90" title={value}>
        {value}
      </div>
    </div>
  );
}

function CopyValueButton({ label, value }: { label: string; value: string }) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      toast.success(t("common.copied"));
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      toast.error(t("dockerManager.copyFailed"));
    }
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 opacity-0 text-muted-foreground transition-opacity hover:text-foreground group-hover/row:opacity-100 focus-visible:opacity-100"
          onClick={copy}
          aria-label={label}
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{copied ? t("common.copied") : label}</TooltipContent>
    </Tooltip>
  );
}

function StateBadge({ state }: { state: string }) {
  const { t } = useTranslation();
  const kind = getDockerStateKind(state);
  const labelKey = getDockerStateLabelKey(state);
  return (
    <Badge
      variant="outline"
      className={cn("max-w-24 px-1.5 text-[0.625rem] leading-4", stateBadgeClass(kind))}
      title={state}
    >
      {t(`dockerManager.stateLabels.${labelKey}`)}
    </Badge>
  );
}

function parseDockerPorts(ports: string) {
  return ports
    .split(",")
    .map((port) => port.trim())
    .filter(Boolean);
}

function formatPort(port: string) {
  if (!port || port === "-") return "-";
  return port.replace(/->/g, " -> ");
}

function formatMemoryUsed(memoryUsage: string) {
  return memoryUsage.split("/")[0]?.trim() || "-";
}

function middleEllipsis(value: string, prefixLength = 20, suffixLength = 8) {
  if (value.length <= prefixLength + suffixLength + 3) return value;
  return `${value.slice(0, prefixLength)}...${value.slice(-suffixLength)}`;
}

function getDockerStateKind(state: string): DockerStateKind {
  const normalized = state.toLowerCase();
  if (normalized === "running") return "running";
  if (normalized === "exited" || normalized === "created") return "stopped";
  if (normalized === "paused" || normalized === "restarting" || normalized === "removing") {
    return "transition";
  }
  if (normalized === "dead") return "danger";
  return "unknown";
}

function getDockerStateLabelKey(state: string) {
  const normalized = state.toLowerCase();
  if (
    normalized === "created" ||
    normalized === "dead" ||
    normalized === "exited" ||
    normalized === "paused" ||
    normalized === "removing" ||
    normalized === "restarting" ||
    normalized === "running"
  ) {
    return normalized;
  }
  return "unknown";
}

function stateBadgeClass(kind: DockerStateKind) {
  switch (kind) {
    case "danger":
      return "border-red-500/35 bg-red-500/10 text-red-300";
    case "running":
      return "border-emerald-500/35 bg-emerald-500/10 text-emerald-300";
    case "transition":
      return "border-amber-500/35 bg-amber-500/10 text-amber-300";
    case "stopped":
      return "border-slate-500/35 bg-slate-500/10 text-slate-300";
    default:
      return "border-border bg-background text-muted-foreground";
  }
}
