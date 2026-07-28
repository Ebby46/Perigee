.button {
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast),
    transform var(--transition-fast),
    box-shadow var(--transition-fast);

  border: none;

  cursor: pointer;

  user-select: none;
}

.button:hover {
  transform: translateY(-1px);

  box-shadow: var(--shadow-md);
}

.button:active {
  transform: scale(.98);
}

.button:disabled {
  cursor: not-allowed;

  opacity: .6;

  box-shadow: none;

  transform: none;
}

.primary {
  background: var(--color-primary);

  color: white;
}

.primary:hover {
  background: var(--color-primary-hover);
}

.secondary {
  background: var(--color-secondary);

  color: white;
}

.secondary:hover {
  background: var(--color-secondary-hover);
}

.outline {
  background: transparent;

  border: 1px solid var(--color-border);

  color: var(--color-text);
}

.outline:hover {
  background: var(--color-surface);
}

.loading {
  pointer-events: none;
}