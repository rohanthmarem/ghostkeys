import { useState } from "react";
import type { Config } from "../lib/types";

interface SettingsPanelProps {
  config: Config;
  onUpdate: (updates: Partial<Config>) => void;
  onReset: () => void;
  disabled?: boolean;
}

interface SliderProps {
  label: string;
  tooltip: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
  disabled?: boolean;
  formatValue?: (value: number) => string;
}

function Tooltip({ text }: { text: string }) {
  return (
    <div className="tooltip">
      {text}
    </div>
  );
}

function Slider({
  label,
  tooltip,
  value,
  min,
  max,
  step = 1,
  unit = "",
  onChange,
  disabled,
  formatValue,
}: SliderProps) {
  const displayValue = formatValue ? formatValue(value) : value;

  return (
    <div className="space-y-2">
      <div className="flex justify-between text-sm">
        <div className="tooltip-container relative">
          <span className="text-ghost-300 cursor-help border-b border-dotted border-ghost-500">
            {label}
          </span>
          <Tooltip text={tooltip} />
        </div>
        <span className="text-ghost-400 font-mono text-xs">
          {displayValue}
          {unit}
        </span>
      </div>
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        disabled={disabled}
        className="w-full h-2 rounded-lg cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      />
    </div>
  );
}

interface ToggleProps {
  label: string;
  tooltip: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

function Toggle({ label, tooltip, checked, onChange, disabled }: ToggleProps) {
  return (
    <div className="flex items-center justify-between">
      <div className="tooltip-container relative">
        <span className="text-ghost-300 text-sm cursor-help border-b border-dotted border-ghost-500">
          {label}
        </span>
        <Tooltip text={tooltip} />
      </div>
      <button
        aria-label={label}
        aria-pressed={checked}
        onClick={() => onChange(!checked)}
        disabled={disabled}
        className={`relative w-11 h-6 rounded-full transition-colors
                   ${checked ? "bg-accent-primary" : "bg-ghost-700"}
                   disabled:opacity-50`}
      >
        <span
          className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform
                     ${checked ? "translate-x-5" : ""}`}
        />
      </button>
    </div>
  );
}

export function SettingsPanel({
  config,
  onUpdate,
  onReset,
  disabled,
}: SettingsPanelProps) {
  const [expandedSection, setExpandedSection] = useState<string | null>(null);

  const toggleSection = (section: string) => {
    setExpandedSection(expandedSection === section ? null : section);
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-ghost-200">Settings</h2>
        <button
          onClick={onReset}
          disabled={disabled}
          className="text-sm text-ghost-400 hover:text-ghost-300 disabled:opacity-50
                     border-b border-dotted border-transparent hover:border-ghost-400"
        >
          Reset to defaults
        </button>
      </div>

      <div className="space-y-3">
        {/* Speed Settings */}
        <div className="rounded-lg border border-ghost-700/50 overflow-hidden">
          <button
            onClick={() => toggleSection("speed")}
            className="w-full px-4 py-3 flex justify-between items-center bg-ghost-800/30 hover:bg-ghost-800/50"
          >
            <h3 className="text-sm font-medium text-ghost-300 tracking-wide">
              Speed
            </h3>
            <svg
              className={`w-4 h-4 text-ghost-400 transition-transform ${expandedSection === "speed" ? "rotate-180" : ""}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div className={`space-y-4 px-4 overflow-hidden transition-all duration-200 ${expandedSection === "speed" ? "py-4 max-h-96" : "max-h-0"}`}>
            <Slider
              label="Base typing speed"
              tooltip="Target words per minute. Higher values type faster."
              value={config.baseWpm}
              min={20}
              max={200}
              unit=" WPM"
              onChange={(v) => onUpdate({ baseWpm: v })}
              disabled={disabled}
            />
            <Slider
              label="Speed variance"
              tooltip="How much the typing speed naturally fluctuates. Higher values feel more human."
              value={config.wpmVariance}
              min={0}
              max={0.5}
              step={0.05}
              formatValue={(v) => `${Math.round(v * 100)}%`}
              onChange={(v) => onUpdate({ wpmVariance: v })}
              disabled={disabled}
            />
          </div>
        </div>

        {/* Mistake Settings */}
        <div className="rounded-lg border border-ghost-700/50 overflow-hidden">
          <button
            onClick={() => toggleSection("mistakes")}
            className="w-full px-4 py-3 flex justify-between items-center bg-ghost-800/30 hover:bg-ghost-800/50"
          >
            <h3 className="text-sm font-medium text-ghost-300 tracking-wide">
              Mistakes
            </h3>
            <svg
              className={`w-4 h-4 text-ghost-400 transition-transform ${expandedSection === "mistakes" ? "rotate-180" : ""}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div className={`space-y-4 px-4 overflow-hidden transition-all duration-200 ${expandedSection === "mistakes" ? "py-4 max-h-96" : "max-h-0"}`}>
            <Slider
              label="Mistake rate"
              tooltip="Probability of making a typo. Set to 0 for perfect typing."
              value={config.mistakeRate}
              min={0}
              max={0.15}
              step={0.01}
              formatValue={(v) => `${Math.round(v * 100)}%`}
              onChange={(v) => onUpdate({ mistakeRate: v })}
              disabled={disabled}
            />
            <Slider
              label="Correction rate"
              tooltip="How often mistakes get corrected with backspace. Lower values leave typos."
              value={config.correctionRate}
              min={0}
              max={1}
              step={0.05}
              formatValue={(v) => `${Math.round(v * 100)}%`}
              onChange={(v) => onUpdate({ correctionRate: v })}
              disabled={disabled}
            />
          </div>
        </div>

        {/* Pause Settings */}
        <div className="rounded-lg border border-ghost-700/50 overflow-hidden">
          <button
            onClick={() => toggleSection("pauses")}
            className="w-full px-4 py-3 flex justify-between items-center bg-ghost-800/30 hover:bg-ghost-800/50"
          >
            <h3 className="text-sm font-medium text-ghost-300 tracking-wide">
              Pauses
            </h3>
            <svg
              className={`w-4 h-4 text-ghost-400 transition-transform ${expandedSection === "pauses" ? "rotate-180" : ""}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div className={`space-y-4 px-4 overflow-hidden transition-all duration-200 ${expandedSection === "pauses" ? "py-4 max-h-96" : "max-h-0"}`}>
            <Slider
              label="Punctuation pause"
              tooltip="Extra delay after punctuation marks like periods and commas."
              value={config.punctuationPause}
              min={0}
              max={1000}
              step={50}
              unit=" ms"
              onChange={(v) => onUpdate({ punctuationPause: v })}
              disabled={disabled}
            />
            <Slider
              label="Paragraph pause"
              tooltip="Extra delay when starting a new line or paragraph."
              value={config.paragraphPause}
              min={0}
              max={3000}
              step={100}
              unit=" ms"
              onChange={(v) => onUpdate({ paragraphPause: v })}
              disabled={disabled}
            />
            <Slider
              label="Thinking pause chance"
              tooltip="Probability of random pauses to simulate thinking."
              value={config.thinkingPauseChance}
              min={0}
              max={0.1}
              step={0.005}
              formatValue={(v) => `${Math.round(v * 100)}%`}
              onChange={(v) => onUpdate({ thinkingPauseChance: v })}
              disabled={disabled}
            />
            <Slider
              label="Thinking pause duration"
              tooltip="How long the thinking pauses last on average."
              value={config.thinkingPauseDuration}
              min={500}
              max={5000}
              step={100}
              unit=" ms"
              onChange={(v) => onUpdate({ thinkingPauseDuration: v })}
              disabled={disabled}
            />
          </div>
        </div>

        {/* Other Settings */}
        <div className="rounded-lg border border-ghost-700/50 overflow-hidden">
          <button
            onClick={() => toggleSection("other")}
            className="w-full px-4 py-3 flex justify-between items-center bg-ghost-800/30 hover:bg-ghost-800/50"
          >
            <h3 className="text-sm font-medium text-ghost-300 tracking-wide">
              Other
            </h3>
            <svg
              className={`w-4 h-4 text-ghost-400 transition-transform ${expandedSection === "other" ? "rotate-180" : ""}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div className={`space-y-4 px-4 overflow-hidden transition-all duration-200 ${expandedSection === "other" ? "py-4 max-h-96" : "max-h-0"}`}>
            <Slider
              label="Countdown"
              tooltip="Seconds to wait before typing starts. Gives you time to focus the target."
              value={config.countdownSeconds}
              min={1}
              max={10}
              unit=" sec"
              onChange={(v) => onUpdate({ countdownSeconds: v })}
              disabled={disabled}
            />
            <Toggle
              label="Burst typing"
              tooltip="Occasionally type faster in short bursts, simulating flow state."
              checked={config.burstTyping}
              onChange={(v) => onUpdate({ burstTyping: v })}
              disabled={disabled}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
