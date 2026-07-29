import { Request, Response, Router } from 'express';
import { CreateVaultSchema, CreateVaultDto } from '../dtos/create-vault.dto';

const router = Router();

/**
 * A mock database or service layer to interact with.
 * In a real application, this would be a proper service class.
 */
const vaultService = {
  create: async (data: CreateVaultDto) => {
    console.log('Creating vault with validated data:', data);
    // Simulate database record creation
    const newVault = {
      id: crypto.randomUUID(),
      ...data,
      createdAt: new Date().toISOString(),
    };
    return newVault;
  },
};

/**
 * POST /vaults
 *
 * Creates a new white-label vault. The request body is validated against
 * the CreateVaultSchema to ensure the managerId is a valid UUID, the name
 * is non-empty, and the markup does not exceed the policy ceiling.
 */
router.post('/vaults', async (req: Request, res: Response) => {
  // 1. Parse and validate the incoming request body using the Zod schema.
  const validationResult = CreateVaultSchema.safeParse(req.body);

  // 2. If validation fails, return a 400 Bad Request response.
  if (!validationResult.success) {
    return res.status(400).json({
      error: 'Validation failed',
      issues: validationResult.error.flatten().fieldErrors,
    });
  }

  // 3. If validation succeeds, proceed with the business logic.
  try {
    const newVault = await vaultService.create(validationResult.data);
    return res.status(201).json(newVault);
  } catch (error) {
    console.error('Failed to create vault:', error);
    return res.status(500).json({ error: 'Internal Server Error' });
  }
});

export default router;