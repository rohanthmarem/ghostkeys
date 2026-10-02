import { useId } from "react";
import type { Config } from "../lib/types";
import { DEFAULT_CONFIG } from "../lib/types";

export const typingStyles: { name: string; config: Config }[] = [
  { name: "Natural", config: DEFAULT_CONFIG },
  { name: "Natural Slow", config: { ...DEFAULT_CONFIG, draftingMode: "slow", baseWpm: 42, wpmVariance: 0.35, mistakeRate: 0.02, correctionRate: 1 } },
  { name: "Natural Fast", config: { ...DEFAULT_CONFIG, draftingMode: "fast", baseWpm: 85, wpmVariance: 0.25, mistakeRate: 0.015, correctionRate: 1 } },
  { name: "Clean", config: { ...DEFAULT_CONFIG, mistakeRate: 0, correctionRate: 1, wpmVariance: 0.15, burstTyping: false } },
  { name: "Quick", config: { ...DEFAULT_CONFIG, baseWpm: 100, mistakeRate: 0, correctionRate: 1, punctuationPause: 150, paragraphPause: 300, thinkingPauseChance: 0, wpmVariance: 0.15 } },
];
export const sameConfig = (a: Config, b: Config) => Object.entries(a).every(([key, value]) => b[key as keyof Config] === value);

function Slider({ label, value, min, max, step = 1, unit = "", onChange, disabled, percentage }: {
  label: string; value: number; min: number; max: number; step?: number; unit?: string;
  onChange: (v: number) => void; disabled?: boolean; percentage?: boolean;
}) {
  const id = useId();
  return <div className="range-row"><div><label htmlFor={id}>{label}</label><output htmlFor={id}>{percentage ? `${Math.round(value * 100)}%` : `${value}${unit}`}</output></div>
    <input id={id} type="range" min={min} max={max} step={step} value={value} onChange={e => onChange(Number(e.target.value))} disabled={disabled} /></div>;
}
export function SettingsPanel({ config, onUpdate, onReset, disabled }: {
  config: Config; onUpdate: (updates: Partial<Config>) => void; onReset: () => void; disabled: boolean;
}) {
  return <div className="settings-content">
    <div className="settings-group">
      <label className="switch-row"><span>Drafting rhythm</span><select aria-label="Drafting rhythm" value={config.draftingMode} disabled={disabled} onChange={e => onUpdate({ draftingMode: e.target.value as Config["draftingMode"] })}><option value="off">Off</option><option value="slow">Slow</option><option value="fast">Fast</option></select></label>
      <Slider label="Start delay" value={config.countdownSeconds} min={3} max={15} unit=" sec" onChange={countdownSeconds => onUpdate({ countdownSeconds })} disabled={disabled} />
      <Slider label="Speed variation" value={config.wpmVariance} min={0} max={0.5} step={0.05} percentage onChange={wpmVariance => onUpdate({ wpmVariance })} disabled={disabled} />
      <label className="switch-row"><span>Faster bursts</span><input className="switch" type="checkbox" checked={config.burstTyping} disabled={disabled} onChange={e => onUpdate({ burstTyping: e.target.checked })} /></label>
    </div>
    <div className="settings-group"><h3>Typos</h3>
      <Slider label="Added typos" value={config.mistakeRate} min={0} max={0.15} step={0.01} percentage onChange={mistakeRate => onUpdate({ mistakeRate })} disabled={disabled} />
      <Slider label="Correction rate" value={config.draftingMode === "off" ? config.correctionRate : 1} min={0} max={1} step={0.05} percentage onChange={correctionRate => onUpdate({ correctionRate })} disabled={disabled || config.mistakeRate === 0 || config.draftingMode !== "off"} />
    </div>
    {config.draftingMode === "off" ? <div className="settings-group"><h3>Pauses</h3>
      <Slider label="Punctuation" value={config.punctuationPause} min={0} max={1000} step={50} unit=" ms" onChange={punctuationPause => onUpdate({ punctuationPause })} disabled={disabled} />
      <Slider label="Paragraphs" value={config.paragraphPause} min={0} max={3000} step={100} unit=" ms" onChange={paragraphPause => onUpdate({ paragraphPause })} disabled={disabled} />
      <Slider label="Thinking pauses" value={config.thinkingPauseChance} min={0} max={0.1} step={0.005} percentage onChange={thinkingPauseChance => onUpdate({ thinkingPauseChance })} disabled={disabled} />
      <Slider label="Thinking pause length" value={config.thinkingPauseDuration} min={500} max={5000} step={100} unit=" ms" onChange={thinkingPauseDuration => onUpdate({ thinkingPauseDuration })} disabled={disabled || config.thinkingPauseChance === 0} />
    </div> : <p className="empty-presets">Pauses follow phrases, sentences, and paragraphs. Brief revisions are corrected before continuing.</p>}
    <button className="text-button" onClick={onReset} disabled={disabled}>Reset typing settings</button>
  </div>;
}
