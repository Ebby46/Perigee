import { CreateVaultSchema, MAX_MARKUP_BPS } from './create-vault.dto';

describe('CreateVaultSchema', () => {
  // Test case for a perfectly valid request body
  it('should validate a correct payload successfully', () => {
    const validPayload = {
      clientName: 'Test Client',
      markupBps: 100, // 1% markup
    };
    const result = CreateVaultSchema.safeParse(validPayload);
    expect(result.success).toBe(true);
  });

  // Test cases for the markup ceiling
  it(`should accept markup equal to the maximum of ${MAX_MARKUP_BPS} bps`, () => {
    const payload = {
      clientName: 'Edge Case Client',
      markupBps: MAX_MARKUP_BPS,
    };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(true);
  });

  it(`should reject markup exceeding the maximum of ${MAX_MARKUP_BPS} bps`, () => {
    const payload = {
      clientName: 'Invalid Client',
      markupBps: MAX_MARKUP_BPS + 1,
    };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe(
        `Markup cannot exceed ${MAX_MARKUP_BPS} basis points.`,
      );
    }
  });

  // Test cases for other invalid inputs
  it('should reject a negative markup', () => {
    const payload = { clientName: 'Negative Markup', markupBps: -1 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Markup cannot be negative.');
    }
  });

  it('should reject a non-integer markup', () => {
    const payload = { clientName: 'Float Markup', markupBps: 150.5 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Markup must be an integer.');
    }
  });

  it('should reject a request with an empty client name', () => {
    const payload = { clientName: '', markupBps: 100 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Client name cannot be empty.');
    }
  });
});