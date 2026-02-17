import { useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ALL_PHASES } from "@/lib/constants";
import { createSkill } from "@/lib/tauri";

interface CreateSkillWizardProps {
  onClose: () => void;
  onCreated: () => void;
}

type WizardStep = "metadata" | "describe" | "generate" | "done";

export function CreateSkillWizard({ onClose, onCreated }: CreateSkillWizardProps) {
  const [step, setStep] = useState<WizardStep>("metadata");
  const [name, setName] = useState("");
  const [phase, setPhase] = useState("code");
  const [tech, setTech] = useState("general");
  const [description, setDescription] = useState("");
  const [output, setOutput] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function handleGenerate() {
    setStep("generate");
    setOutput("Starting generation...\n");
    setError(null);

    const unlistenOutput = await listen<string>("create-skill-output", (event) => {
      setOutput((prev) => prev + event.payload);
    });
    const unlistenDone = await listen<string>("create-skill-done", () => {
      setStep("done");
      unlistenOutput();
      unlistenDone();
      unlistenError();
    });
    const unlistenError = await listen<string>("create-skill-error", (event) => {
      setError(event.payload);
      setStep("describe");
      unlistenOutput();
      unlistenDone();
      unlistenError();
    });

    try {
      await createSkill(name, phase, tech, description);
    } catch (e) {
      setError(String(e));
      setStep("describe");
      unlistenOutput();
      unlistenDone();
      unlistenError();
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-full max-w-lg rounded-xl border bg-card p-6 shadow-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-bold">Create Skill</h3>
          <button
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground text-lg"
          >
            &times;
          </button>
        </div>

        {step === "metadata" && (
          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Name (kebab-case)
              </label>
              <Input
                value={name}
                onChange={(e) =>
                  setName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ""))
                }
                placeholder="my-agent"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Phase
              </label>
              <select
                value={phase}
                onChange={(e) => setPhase(e.target.value)}
                className="w-full h-9 rounded-md border bg-transparent px-3 text-sm"
              >
                {ALL_PHASES.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Tech
              </label>
              <Input
                value={tech}
                onChange={(e) => setTech(e.target.value)}
                placeholder="general"
              />
            </div>
            <Button
              className="w-full"
              disabled={!name.trim() || !tech.trim()}
              onClick={() => setStep("describe")}
            >
              Next
            </Button>
          </div>
        )}

        {step === "describe" && (
          <div className="space-y-3">
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                What should this agent do?
              </label>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="w-full rounded-md border bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground resize-none h-32"
                placeholder="Describe the agent's purpose and capabilities..."
              />
            </div>
            {error && (
              <p className="text-xs text-destructive">{error}</p>
            )}
            <div className="flex gap-2">
              <Button
                variant="outline"
                className="flex-1"
                onClick={() => setStep("metadata")}
              >
                Back
              </Button>
              <Button
                className="flex-1"
                disabled={!description.trim()}
                onClick={handleGenerate}
              >
                Generate with Claude
              </Button>
            </div>
          </div>
        )}

        {step === "generate" && (
          <div className="space-y-3">
            <pre className="text-xs bg-muted rounded-md p-3 h-48 overflow-auto whitespace-pre-wrap font-mono">
              {output}
            </pre>
            <p className="text-xs text-muted-foreground text-center">
              Generating agent...
            </p>
          </div>
        )}

        {step === "done" && (
          <div className="space-y-3">
            <pre className="text-xs bg-muted rounded-md p-3 h-48 overflow-auto whitespace-pre-wrap font-mono">
              {output}
            </pre>
            <p className="text-sm text-green-400 text-center">
              Skill created successfully!
            </p>
            <Button className="w-full" onClick={onCreated}>
              Done
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
