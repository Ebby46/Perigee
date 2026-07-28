import { z } from 'zod';

/**
 * The maximum permissible fee markup in basis points (bps).
 * 100 bps = 1%.
 * A value of 500 means a 5% maximum markup.
 */
export const MAX_MARKUP_BPS = 500;

/**
 * Zod schema for validating the `create-vault` request body.
 *
 * It ensures that:
 * - `clientName` is a non-empty string.
 * - `markupBps` is an integer between 0 and `MAX_MARKUP_BPS`.
 */
export const CreateVaultSchema = z.object({
  clientName: z.string().min(1, { message: 'Client name cannot be empty.' }),
  markupBps: z
    .number()
    .int({ message: 'Markup must be an integer.' })
    .min(0, { message: 'Markup cannot be negative.' })
    .max(MAX_MARKUP_BPS, {
      message: `Markup cannot exceed ${MAX_MARKUP_BPS} basis points.`,
    }),
});

/**
 * Type definition for the validated `create-vault` request body.
 */
export type CreateVaultDto = z.infer<typeof CreateVaultSchema>;