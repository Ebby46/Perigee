import { CreateVaultSchema, MAX_MARKUP_BPS } from './create-vault.dto';

describe('CreateVaultSchema', () => {
  const validManagerId = '550e8400-e29b-41d4-a716-446655440000';

  it('should validate a correct payload successfully', () => {
    const validPayload = {
      managerId: validManagerId,
      name: 'Test Client',
      markupBps: 100,
    };
    const result = CreateVaultSchema.safeParse(validPayload);
    expect(result.success).toBe(true);
  });

  it(`should accept markup equal to the maximum of ${MAX_MARKUP_BPS} bps`, () => {
    const payload = {
      managerId: validManagerId,
      name: 'Edge Case Client',
      markupBps: MAX_MARKUP_BPS,
    };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(true);
  });

  it(`should reject markup exceeding the maximum of ${MAX_MARKUP_BPS} bps`, () => {
    const payload = {
      managerId: validManagerId,
      name: 'Invalid Client',
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

  it('should reject a negative markup', () => {
    const payload = { managerId: validManagerId, name: 'Negative Markup', markupBps: -1 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Markup cannot be negative.');
    }
  });

  it('should reject a non-integer markup', () => {
    const payload = { managerId: validManagerId, name: 'Float Markup', markupBps: 150.5 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Markup must be an integer.');
    }
  });

  it('should reject a request with an empty name', () => {
    const payload = { managerId: validManagerId, name: '', markupBps: 100 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('Vault name cannot be empty.');
    }
  });

  it('should reject an invalid managerId', () => {
    const payload = { managerId: 'not-a-uuid', name: 'Client', markupBps: 100 };
    const result = CreateVaultSchema.safeParse(payload);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].message).toBe('managerId must be a valid UUID.');
    }
  });
});
