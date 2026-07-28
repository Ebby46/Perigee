import React from "react";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: Parameters<typeof clsx>) {
  return twMerge(clsx(inputs));
}

type BadgeVariant = "default" | "success" | "warning" | "danger" | "info";

interface BadgeProps {
  variant?: BadgeVariant;
  children: React.ReactNode;
  className?: string;
}

const variantClasses: Record<BadgeVariant, string> = {
  default: "bg-slate-700 text-slate-300",
  success: "bg-emerald-900/60 text-emerald-300",
  warning: "bg-amber-900/60 text-amber-300",
  danger:  "bg-rose-900/60 text-rose-300",
  info:    "bg-sky-900/60 text-sky-300",
};

/**
 * Standardized Badge / pill component.
 *
 * Resolves WEB-27 (#113): component library not standardized.
 * Use for status labels, resource budget indicators, and severity tags.
 *
 * @example
 * <Badge variant="success">Passed</Badge>
 * <Badge variant="danger">Over Budget</Badge>
 */
export function Badge({ variant = "default", children, className }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
        variantClasses[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}
