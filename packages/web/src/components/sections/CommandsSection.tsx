import { SectionHeader } from "@/components/ui/SectionHeader";
import { TerminalBlock } from "@/components/ui/TerminalBlock";

export function CommandsSection() {
  return (
    <section id="commands" className="py-20 md:py-28">
      <div className="max-w-4xl mx-auto px-4">
        <SectionHeader
          label="09 | CLI Commands"
          title="Simple, powerful commands"
          subtitle="Manage your entire skills setup from the terminal. Install, search, plan, and deploy with one tool."
        />

        <TerminalBlock title="Terminal">
          <div className="space-y-1">
            <p className="text-text-subtle"># Setup</p>
            <p><span className="text-accent">forja</span> <span className="text-text">init</span>                          <span className="text-text-subtle"># Initialize registry</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">doctor</span>                        <span className="text-text-subtle"># Health check</span></p>
            <p></p>
            <p className="text-text-subtle"># Browse</p>
            <p><span className="text-accent">forja</span> <span className="text-text">phases</span>                        <span className="text-text-subtle"># Show the 5 workflow phases</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">list</span> <span className="text-scan">--available</span>              <span className="text-text-subtle"># Show all available skills</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">search typescript</span>             <span className="text-text-subtle"># Search by keyword</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">info code/rust/feature</span>        <span className="text-text-subtle"># Skill details</span></p>
            <p></p>
            <p className="text-text-subtle"># Install & Manage</p>
            <p><span className="text-accent">forja</span> <span className="text-text">install test/tdd/workflow</span>    <span className="text-text-subtle"># Install a skill</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">install</span> <span className="text-scan">--all</span>                <span className="text-text-subtle"># Install everything</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">uninstall test/tdd/workflow</span>  <span className="text-text-subtle"># Remove a skill</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">update</span>                        <span className="text-text-subtle"># Update the registry</span></p>
            <p></p>
            <p className="text-text-subtle"># Plan & Execute</p>
            <p><span className="text-accent">forja</span> <span className="text-text">plan &quot;add auth to the API&quot;</span>    <span className="text-text-subtle"># Create implementation plan</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">execute</span>                       <span className="text-text-subtle"># Execute the latest plan</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">task &quot;fix login bug&quot;</span>          <span className="text-text-subtle"># Run task directly</span></p>
            <p></p>
            <p className="text-text-subtle"># Teams</p>
            <p><span className="text-accent">forja</span> <span className="text-text">team preset full-product</span>    <span className="text-text-subtle"># Create team from preset</span></p>
            <p><span className="text-accent">forja</span> <span className="text-text">team list</span>                    <span className="text-text-subtle"># List configured teams</span></p>
          </div>
        </TerminalBlock>
      </div>
    </section>
  );
}
