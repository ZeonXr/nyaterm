import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { NumberInput } from "@/components/ui/number-input";
import { Select, SelectContent, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";


function SettingMeta({ label, desc }: { label: string; desc?: string }) {
  return (
    <div className="min-w-0">
      <Label className="font-medium text-sm">{label}</Label>
      {desc && <p className="text-xs text-muted-foreground">{desc}</p>}
    </div>
  );
}

export function SettingRow({
  label,
  desc,
  children,
}: {
  label: string;
  desc?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid gap-3 min-[560px]:grid-cols-[minmax(10rem,15rem)_minmax(0,1fr)] min-[560px]:items-start">
      <SettingMeta label={label} desc={desc} />
      <div className="flex min-w-0 max-w-full items-center gap-2 justify-self-end min-[560px]:justify-end min-[560px]:justify-self-stretch">
        {children}
      </div>
    </div>
  );
}

export function SettingInput({
  label,
  desc,
  ...inputProps
}: { label: string; desc?: string } & React.ComponentProps<typeof Input>) {
  return (
    <div className="grid gap-3 min-[560px]:grid-cols-[minmax(10rem,15rem)_minmax(0,1fr)] min-[560px]:items-start">
      <SettingMeta label={label} desc={desc} />
      <div className="min-w-0">
        <Input className="w-full text-sm" {...inputProps} />
      </div>
    </div>
  );
}

export function SettingNumberInput({
  label,
  desc,
  value,
  onChange,
  min,
  max,
  step,
  className,
}: {
  label: string;
  desc?: string;
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
  className?: string;
}) {
  return (
    <div className="grid gap-3 min-[560px]:grid-cols-[minmax(10rem,15rem)_minmax(0,1fr)] min-[560px]:items-start">
      <SettingMeta label={label} desc={desc} />
      <div className="min-w-0">
        <NumberInput
          value={value}
          onChange={onChange}
          min={min}
          max={max}
          step={step}
          className={className}
        />
      </div>
    </div>
  );
}

export function SettingSelect({
  label,
  desc,
  value,
  onValueChange,
  children,
}: {
  label: string;
  desc?: string;
  value: string;
  onValueChange: (v: string) => void;
  children: React.ReactNode;
}) {
  return (
    <div className="grid gap-3 min-[560px]:grid-cols-[minmax(10rem,15rem)_minmax(0,1fr)] min-[560px]:items-start">
      <SettingMeta label={label} desc={desc} />
      <div className="min-w-0">
        <Select value={value} onValueChange={onValueChange}>
          <SelectTrigger className="w-full text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>{children}</SelectContent>
        </Select>
      </div>
    </div>
  );
}

export function SettingSwitch({
  checked,
  disabled,
  onChange,
}: {
  checked: boolean;
  disabled?: boolean;
  onChange: (v: boolean) => void;
}) {
  return <Switch checked={checked} disabled={disabled} onCheckedChange={onChange} />;
}
