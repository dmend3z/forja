import { Button } from "@/components/ui/button";

interface InstallButtonProps {
  installed: boolean;
  loading: boolean;
  onInstall: () => void;
  onUninstall: () => void;
}

export function InstallButton({
  installed,
  loading,
  onInstall,
  onUninstall,
}: InstallButtonProps) {
  return (
    <Button
      variant={installed ? "destructive" : "default"}
      disabled={loading}
      onClick={installed ? onUninstall : onInstall}
    >
      {loading ? "..." : installed ? "Uninstall" : "Install"}
    </Button>
  );
}
