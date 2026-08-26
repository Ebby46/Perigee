export interface StrategyPhase {
  id: string;
  phase: string;
  marketCondition: string;
  action: string;
  description: string;
}

export const strategyPhases: StrategyPhase[] = [
  {
    id: "accumulation",
    phase: "Accumulation",
    marketCondition: "Bullish Reversal",
    action: "Increase exposure",
    description:
      "Gradually accumulate assets during early bullish confirmation.",
  },
  {
    id: "markup",
    phase: "Markup",
    marketCondition: "Bull Market",
    action: "Hold / Scale In",
    description:
      "Ride the upward trend while managing risk.",
  },
  {
    id: "distribution",
    phase: "Distribution",
    marketCondition: "Bearish Reversal",
    action: "Reduce exposure",
    description:
      "Take profits as momentum weakens.",
  },
  {
    id: "markdown",
    phase: "Markdown",
    marketCondition: "Bear Market",
    action: "Preserve capital",
    description:
      "Limit exposure and protect capital during sustained declines.",
  },
];