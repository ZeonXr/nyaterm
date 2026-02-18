import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

/** Footer bar showing current time. */
export default function StatusBar() {
  const { i18n } = useTranslation();
  const [time, setTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  const locale = i18n.language === "zh-CN" ? "zh-CN" : "en-US";
  const formattedTime = time.toLocaleTimeString(locale, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: i18n.language !== "zh-CN",
  });

  return (
    <footer
      className="h-7 text-white flex items-center justify-between px-3 text-[11px] select-none shrink-0"
      style={{ backgroundColor: "var(--df-primary)" }}
    >
      <div className="flex items-center gap-4 h-full"></div>
      <div className="flex items-center gap-4 h-full">
        <div className="flex items-center gap-1 bg-black/20 px-3 h-full">
          <span className="font-bold">{formattedTime}</span>
        </div>
      </div>
    </footer>
  );
}
