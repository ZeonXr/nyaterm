import { useTranslation } from "react-i18next";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

export interface DockerPendingAction {
  title: string;
  description: string;
  command: string;
  run: () => Promise<void>;
}

interface DockerConfirmDialogProps {
  action: DockerPendingAction | null;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
}

export default function DockerConfirmDialog({
  action,
  onConfirm,
  onOpenChange,
}: DockerConfirmDialogProps) {
  const { t } = useTranslation();

  return (
    <AlertDialog open={Boolean(action)} onOpenChange={onOpenChange}>
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>{action?.title}</AlertDialogTitle>
          <AlertDialogDescription>{action?.description}</AlertDialogDescription>
        </AlertDialogHeader>
        {action?.command ? (
          <div className="rounded-md border bg-muted/40 px-2 py-1.5 font-mono text-xs break-all">
            {action.command}
          </div>
        ) : null}
        <AlertDialogFooter>
          <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            {t("common.confirm")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
