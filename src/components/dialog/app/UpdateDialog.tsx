import { getVersion } from "@tauri-apps/api/app";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdCheckCircle, MdError, MdRestartAlt } from "react-icons/md";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import type { UpdateInfo, UpdateProgress, UpdateStatus } from "@/lib/updater";
import { checkForUpdate, downloadAndInstallUpdate, relaunchApp } from "@/lib/updater";

interface UpdateDialogProps {
  open: boolean;
  onClose: () => void;
  onUpdateFound?: (info: UpdateInfo) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / k ** i).toFixed(1)} ${sizes[i]}`;
}

export default function UpdateDialog({ open, onClose, onUpdateFound }: UpdateDialogProps) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<UpdateStatus>("checking");
  const [progress, setProgress] = useState<UpdateProgress>({ downloaded: 0, total: 0 });
  const [error, setError] = useState<string>("");
  const [currentVersion, setCurrentVersion] = useState("");
  const [localUpdateInfo, setLocalUpdateInfo] = useState<UpdateInfo | null>(null);
  const isUpdating = useRef(false);
  const onUpdateFoundRef = useRef(onUpdateFound);
  onUpdateFoundRef.current = onUpdateFound;

  useEffect(() => {
    if (!open) return;

    getVersion()
      .then(setCurrentVersion)
      .catch(() => {});
    setStatus("checking");
    setProgress({ downloaded: 0, total: 0 });
    setError("");
    setLocalUpdateInfo(null);
    isUpdating.current = false;

    let cancelled = false;
    checkForUpdate()
      .then((info) => {
        if (cancelled) return;
        if (info) {
          setLocalUpdateInfo(info);
          setStatus("available");
          onUpdateFoundRef.current?.(info);
        } else {
          setStatus("idle");
        }
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : String(err));
        setStatus("error");
      });

    return () => {
      cancelled = true;
    };
  }, [open]);

  const handleUpdate = useCallback(async () => {
    if (isUpdating.current) return;
    isUpdating.current = true;
    setStatus("downloading");
    setError("");

    try {
      await downloadAndInstallUpdate((p) => {
        setProgress(p);
      });
      setStatus("ready");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus("error");
      isUpdating.current = false;
    }
  }, []);

  const handleRelaunch = useCallback(async () => {
    try {
      await relaunchApp();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus("error");
    }
  }, []);

  const canClose =
    status === "checking" || status === "available" || status === "idle" || status === "error";
  const percent = progress.total > 0 ? Math.round((progress.downloaded / progress.total) * 100) : 0;

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v && canClose) onClose();
      }}
    >
      <DialogContent
        className="w-[420px] sm:max-w-[420px]"
        showCloseButton={canClose}
        onPointerDownOutside={(e) => {
          if (!canClose) e.preventDefault();
        }}
        onEscapeKeyDown={(e) => {
          if (!canClose) e.preventDefault();
        }}
      >
        {status === "checking" && (
          <DialogHeader>
            <DialogTitle>{t("updater.checking")}</DialogTitle>
          </DialogHeader>
        )}

        {status === "idle" && (
          <>
            <DialogHeader>
              <DialogTitle>{t("updater.noUpdate")}</DialogTitle>
              <DialogDescription className="text-xs pt-1">
                {t("updater.currentVersion")}: v{currentVersion}
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" size="sm" onClick={onClose}>
                {t("common.close")}
              </Button>
            </DialogFooter>
          </>
        )}

        {status === "available" && localUpdateInfo && (
          <>
            <DialogHeader>
              <DialogTitle>{t("updater.newVersionAvailable")}</DialogTitle>
              <DialogDescription className="space-y-1 pt-1">
                <span className="block text-xs">
                  {t("updater.currentVersion")}: v{currentVersion}
                </span>
                <span className="block text-xs">
                  {t("updater.newVersion")}: v{localUpdateInfo.version}
                </span>
                {localUpdateInfo.date && (
                  <span className="block text-xs">
                    {t("updater.releaseDate")}:{" "}
                    {new Date(localUpdateInfo.date).toLocaleDateString()}
                  </span>
                )}
              </DialogDescription>
            </DialogHeader>

            {localUpdateInfo.body && (
              <div className="max-h-[200px] overflow-y-auto rounded-md border p-3">
                <p className="text-xs font-medium mb-1.5 text-muted-foreground">
                  {t("updater.releaseNotes")}
                </p>
                <div className="text-xs leading-relaxed whitespace-pre-wrap text-foreground">
                  {localUpdateInfo.body}
                </div>
              </div>
            )}

            <DialogFooter>
              <Button variant="outline" size="sm" onClick={onClose}>
                {t("common.cancel")}
              </Button>
              <Button size="sm" onClick={handleUpdate}>
                {t("updater.updateNow")}
              </Button>
            </DialogFooter>
          </>
        )}

        {status === "downloading" && (
          <>
            <DialogHeader>
              <DialogTitle>{t("updater.downloading")}</DialogTitle>
              <DialogDescription>
                {formatBytes(progress.downloaded)} /{" "}
                {progress.total > 0 ? formatBytes(progress.total) : "..."}
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-2 py-2">
              <Progress value={percent} className="h-2" />
              <p className="text-xs text-center text-muted-foreground">{percent}%</p>
            </div>
          </>
        )}

        {status === "ready" && (
          <>
            <DialogHeader className="items-center pt-2">
              <MdCheckCircle className="text-4xl text-green-500 mb-2" />
              <DialogTitle>{t("updater.readyToRestart")}</DialogTitle>
            </DialogHeader>
            <DialogFooter className="pt-2">
              <Button size="sm" onClick={handleRelaunch}>
                <MdRestartAlt className="mr-1.5" />
                {t("updater.restartNow")}
              </Button>
            </DialogFooter>
          </>
        )}

        {status === "error" && (
          <>
            <DialogHeader className="items-center pt-2">
              <MdError className="text-4xl text-red-500 mb-2" />
              <DialogTitle>{t("updater.updateFailed")}</DialogTitle>
              <DialogDescription className="text-xs break-all">{error}</DialogDescription>
            </DialogHeader>
            <DialogFooter className="pt-2">
              <Button variant="outline" size="sm" onClick={onClose}>
                {t("common.cancel")}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
