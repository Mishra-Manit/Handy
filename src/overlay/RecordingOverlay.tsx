import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import "./RecordingOverlay.css";
import i18n, { syncLanguageFromSettings } from "@/i18n";
import { getLanguageDirection } from "@/lib/utils/rtl";

type OverlayState = "recording" | "transcribing" | "processing";

const BAR_COUNT = 11;

// Pre-compute Gaussian envelope so center bars are taller, edges taper off
const ENVELOPE = Array.from({ length: BAR_COUNT }, (_, i) => {
  const center = (BAR_COUNT - 1) / 2;
  const distance = Math.abs(i - center) / center;
  return 0.3 + 0.7 * Math.cos(distance * Math.PI * 0.5);
});

const RecordingOverlay: React.FC = () => {
  const { t } = useTranslation();
  const [isVisible, setIsVisible] = useState(false);
  const [state, setState] = useState<OverlayState>("recording");
  const [levels, setLevels] = useState<number[]>(Array(16).fill(0));
  const smoothedLevelsRef = useRef<number[]>(Array(16).fill(0));
  const direction = getLanguageDirection(i18n.language);

  useEffect(() => {
    const setupEventListeners = async () => {
      const unlistenShow = await listen("show-overlay", async (event) => {
        await syncLanguageFromSettings();
        const overlayState = event.payload as OverlayState;
        setState(overlayState);
        setIsVisible(true);
      });

      const unlistenHide = await listen("hide-overlay", () => {
        setIsVisible(false);
      });

      const unlistenLevel = await listen<number[]>("mic-level", (event) => {
        const newLevels = event.payload as number[];

        const smoothed = smoothedLevelsRef.current.map((prev, i) => {
          const target = newLevels[i] || 0;
          return prev * 0.7 + target * 0.3;
        });

        smoothedLevelsRef.current = smoothed;
        setLevels(smoothed.slice(0, 6));
      });

      return () => {
        unlistenShow();
        unlistenHide();
        unlistenLevel();
      };
    };

    setupEventListeners();
  }, []);

  // Mirror 6 levels into 11 symmetric bars: [l5,l4,l3,l2,l1, l0, l1,l2,l3,l4,l5]
  const mirroredLevels = [...levels.slice(1).reverse(), ...levels];

  return (
    <div
      dir={direction}
      className={`recording-overlay ${isVisible ? "fade-in" : ""}`}
    >
      {state === "recording" && (
        <div className="bars-container">
          {mirroredLevels.map((v, i) => {
            const weight = ENVELOPE[i] ?? 0.3;
            const boosted = Math.min(1, v * weight * 2.5);
            const height = Math.max(
              3,
              Math.min(22, 3 + Math.pow(boosted, 0.35) * 19),
            );
            const barOpacity = 0.4 + Math.min(0.55, boosted * 0.65);
            return (
              <div
                key={i}
                className="bar"
                style={{
                  height: `${height}px`,
                  opacity: barOpacity,
                }}
              />
            );
          })}
        </div>
      )}
      {state === "transcribing" && (
        <div className="transcribing-text">{t("overlay.transcribing")}</div>
      )}
      {state === "processing" && (
        <div className="transcribing-text">{t("overlay.processing")}</div>
      )}
    </div>
  );
};

export default RecordingOverlay;
