import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdAdd, MdDelete, MdEdit } from "react-icons/md";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { SavedPassword } from "@/types/global";

export function PasswordManagementTab() {
  const { t } = useTranslation();
  const [passwords, setPasswords] = useState<SavedPassword[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editPassword, setEditPassword] = useState("");
  const [editHasPassword, setEditHasPassword] = useState(false);
  const [isNew, setIsNew] = useState(false);
  const [deletingEntry, setDeletingEntry] = useState<SavedPassword | null>(null);

  const loadPasswords = useCallback(async () => {
    try {
      const result = await invoke<SavedPassword[]>("get_saved_passwords");
      setPasswords(result);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    loadPasswords();
  }, [loadPasswords]);

  const resetEdit = () => {
    setEditingId(null);
    setEditName("");
    setEditPassword("");
    setEditHasPassword(false);
    setIsNew(false);
  };

  const handleAdd = () => {
    resetEdit();
    setEditingId("__new__");
    setIsNew(true);
  };

  const handleEdit = (entry: SavedPassword) => {
    setEditingId(entry.id);
    setEditName(entry.name);
    setEditPassword("");
    setEditHasPassword(entry.has_password || false);
    setIsNew(false);
  };

  const handleSave = async () => {
    if (!editName.trim()) return;
    if (isNew && !editPassword) return;
    try {
      await invoke("save_password", {
        entry: {
          id: isNew ? "" : editingId,
          name: editName.trim(),
          password: editPassword || undefined,
        },
      });
      resetEdit();
      await loadPasswords();
    } catch { /* ignore */ }
  };

  const handleDeleteConfirm = async () => {
    if (!deletingEntry) return;
    try {
      await invoke("delete_password", { id: deletingEntry.id });
      await loadPasswords();
    } catch { /* ignore */ }
    setDeletingEntry(null);
  };

  const PasswordForm = ({ isEditing }: { isEditing: boolean }) => (
    <div className="p-3 border-b space-y-2.5 bg-accent/30">
      <Input
        placeholder={t("passwordManager.namePlaceholder")}
        className="text-xs h-8"
        value={editName}
        onChange={(e) => setEditName(e.target.value)}
        autoFocus
      />
      <Input
        type="password"
        placeholder={
          isEditing && editHasPassword
            ? t("passwordManager.passwordUnchanged")
            : t("passwordManager.passwordPlaceholder")
        }
        className="text-xs h-8"
        value={editPassword}
        onChange={(e) => setEditPassword(e.target.value)}
      />
      <div className="flex justify-end gap-1.5 pt-0.5">
        <Button variant="outline" size="sm" className="h-7 px-3 text-xs" onClick={resetEdit}>
          {t("common.cancel")}
        </Button>
        <Button
          size="sm"
          className="h-7 px-3 text-xs"
          onClick={handleSave}
          disabled={!editName.trim() || (!isEditing && !editPassword)}
        >
          {t("common.save")}
        </Button>
      </div>
    </div>
  );

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label className="font-medium text-sm">{t("passwordManager.title")}</Label>
          <Button
            variant="ghost"
            size="sm"
            className="text-primary h-7 px-2 text-xs"
            onClick={handleAdd}
            disabled={editingId !== null}
          >
            <MdAdd className="text-base mr-1" /> {t("passwordManager.add")}
          </Button>
        </div>

        <div className="border rounded-md overflow-hidden">
          {isNew && editingId === "__new__" && <PasswordForm isEditing={false} />}

          {passwords.map((entry) => (
            <div key={entry.id}>
              {editingId === entry.id && !isNew ? (
                <PasswordForm isEditing={true} />
              ) : (
                <div className="flex items-center gap-2 px-3 py-2.5 border-b last:border-0 hover:bg-accent transition-colors">
                  <span className="flex-1 text-xs truncate">{entry.name}</span>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    onClick={() => handleEdit(entry)}
                    disabled={editingId !== null}
                  >
                    <MdEdit className="text-base" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    className="text-destructive hover:bg-destructive/10"
                    onClick={() => setDeletingEntry(entry)}
                    disabled={editingId !== null}
                  >
                    <MdDelete className="text-base" />
                  </Button>
                </div>
              )}
            </div>
          ))}

          {passwords.length === 0 && !isNew && (
            <div className="text-center py-6 text-xs text-muted-foreground">
              {t("passwordManager.noPasswords")}
            </div>
          )}
        </div>
      </div>

      <Dialog open={deletingEntry !== null} onOpenChange={(v) => !v && setDeletingEntry(null)}>
        <DialogContent showCloseButton={false} className="max-w-sm">
          <DialogHeader>
            <DialogTitle>{t("passwordManager.deleteTitle")}</DialogTitle>
            <DialogDescription>
              {t("passwordManager.deleteConfirm", { name: deletingEntry?.name })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeletingEntry(null)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={handleDeleteConfirm}>
              {t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
