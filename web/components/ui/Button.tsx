import React from "react";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

// ---------------------------------------------------------------------------
// Shared utility — mirrors shadcn/ui convention (WEB-27, #113)
// ---------------------------------------------------------------------------
function cn(...inputs: Parameters<typeof clsx>) {
  return twMerge(clsx(inputs));
}

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    "bg-sky-600 text-white hover:bg-sky-500 focus-visible:ring-sky-500 disabled:bg-sky-900 disabled:text-sky-400",
  secondary:
    "bg-slate-700 text-slate-100 hover:bg-slate-600 focus-visible:ring-slate-500 disabled:bg-slate-800 disabled:text-slate-500",
  ghost:
    "bg-transparent text-slate-300 hover:bg-slate-800 hover:text-slate-100 focus-visible:ring-slate-500 disabled:text-slate-600",
  danger:
    "bg-rose-700 text-white hover:bg-rose-600 focus-visible:ring-rose-500 disabled:bg-rose-900 disabled:text-rose-400",
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: "h-7 px-2.5 text-xs gap-1.5",
  md: "h-9 px-4 text-sm gap-2",
  lg: "h-11 px-6 text-base gap-2.5",
};

/**
 * Standardized Button component for the Perigee component library.
 *
 * Resolves WEB-27 (#113): component library not standardized.
 * All interactive buttons in the app should use this component
 * instead of ad-hoc <button> elements to ensure consistent styling,
 * focus management, and loading/disabled states.
 *
 * @example
 * <Button variant="primary" onClick={handleClick}>Connect Wallet</Button>
 * <Button variant="secondary" size="sm" isLoading>Analyzing...</Button>
 */
export function Button({
  variant = "primary",
  size = "md",
  isLoading = false,
  leftIcon,
  rightIcon,
  children,
  className,
  disabled,
  ...rest
}: ButtonProps) {
  const isDisabled = disabled || isLoading;

  return (
    <button
      {...rest}
      disabled={isDisabled}
      aria-busy={isLoading}
      className={cn(
        // Base
        "inline-flex items-center justify-center font-medium rounded-md transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950",
        "disabled:pointer-events-none disabled:cursor-not-allowed",
        // Variant & size
        variantClasses[variant],
        sizeClasses[size],
        className,
      )}
    >
      {isLoading ? (
        <svg
          className="animate-spin h-4 w-4 shrink-0"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
          />
        </svg>
      ) : (
        leftIcon && <span className="shrink-0">{leftIcon}</span>
      )}
      <span>{children}</span>
      {!isLoading && rightIcon && <span className="shrink-0">{rightIcon}</span>}
    </button>
  );
}
