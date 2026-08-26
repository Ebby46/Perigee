import Mermaid from "@/components/Mermaid";

import { StrategyPhaseTable } from "./StrategyPhaseTable";

export function StrategyPhaseDiagram() {
  return (
    <section
      aria-labelledby="strategy-phases-heading"
    >
      <h2 id="strategy-phases-heading">
        Strategy Phases
      </h2>

      <figure>
        <Mermaid />

        <figcaption>
          Visual representation of the
          market strategy lifecycle.
        </figcaption>
      </figure>

      <div className="mt-8">
        <h3>Accessible Alternative</h3>

        <p className="mb-4">
          The following table presents the
          same information contained in the
          strategy diagram.
        </p>

        <StrategyPhaseTable />
      </div>
    </section>
  );
}