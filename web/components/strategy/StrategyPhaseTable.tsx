import { strategyPhases } from "./strategyPhases";

export function StrategyPhaseTable() {
  return (
    <div className="overflow-x-auto">
      <table className="min-w-full border-collapse border border-gray-200">
        <caption className="sr-only">
          Strategy phases explained in tabular form.
        </caption>

        <thead>
          <tr>
            <th>Phase</th>
            <th>Market</th>
            <th>Recommended Action</th>
            <th>Description</th>
          </tr>
        </thead>

        <tbody>
          {strategyPhases.map((phase) => (
            <tr key={phase.id}>
              <td>{phase.phase}</td>
              <td>{phase.marketCondition}</td>
              <td>{phase.action}</td>
              <td>{phase.description}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}