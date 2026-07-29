import { z } from 'zod';

/**
 * The maximum permissible fee markup in basis points (bps).
 * 100 bps = 1%.
 * A value of 500 means a 5% maximum markup.
 */
export const MAX_MARKUP_BPS = 500;

export const CreateVaultSchema = z.object({
  managerId: z.string().uuid({ message: 'managerId must be a valid UUID.' }),
  name: z.string().min(1, { message: 'Vault name cannot be empty.' }),
  markupBps: z
    .number()
    .int({ message: 'Markup must be an integer.' })
    .min(0, { message: 'Markup cannot be negative.' })
    .max(MAX_MARKUP_BPS, {
      message: `Markup cannot exceed ${MAX_MARKUP_BPS} basis points.`,
    }),
});

export type CreateVaultDto = z.infer<typeof CreateVaultSchema>;
