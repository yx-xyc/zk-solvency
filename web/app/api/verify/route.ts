import { execFile } from 'child_process'
import { promisify } from 'util'
import path from 'path'

const execFileAsync = promisify(execFile)

// process.cwd() = web/ when running `npm run dev` from web/
const REPO_ROOT = path.join(process.cwd(), '..')

export async function POST(req: Request) {
  let userId: unknown
  try {
    const body = await req.json()
    userId = body.userId
  } catch {
    return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
  }

  if (typeof userId !== 'number' || !Number.isInteger(userId) || userId < 0) {
    return Response.json({ error: 'userId must be a non-negative integer' }, { status: 400 })
  }

  const binary    = path.join(REPO_ROOT, 'target', 'debug', 'inclusion')
  const usersFile = path.join(REPO_ROOT, 'data', 'users.json')
  const proofFile = path.join(REPO_ROOT, 'proof.json')

  try {
    const { stdout } = await execFileAsync(
      binary,
      ['--user-id', String(userId), '--users-file', usersFile, '--proof-file', proofFile],
      { timeout: 10_000 }
    )

    const balance    = Number(stdout.match(/balance\s*:\s*(\d+)/)?.[1])
    const leafHash   = stdout.match(/leaf_hash:\s*(0x[0-9a-f]+)/)?.[1]
    const merkleRoot = stdout.match(/merkle_root \(recomputed\):\s*(0x[0-9a-f]+)/)?.[1]
    const proofDepth = Number(stdout.match(/proof_depth:\s*(\d+)/)?.[1])
    const verified   = stdout.includes('verification: OK')

    return Response.json({ verified, userId, balance, leafHash, merkleRoot, proofDepth })
  } catch (err: unknown) {
    const e = err as { stderr?: string; code?: string }
    const stderr = e.stderr ?? ''

    if (e.code === 'ENOENT') {
      return Response.json(
        { error: 'Inclusion binary not found — run: cargo build -p inclusion' },
        { status: 500 }
      )
    }
    if (stderr.includes('not found')) {
      return Response.json(
        { verified: false, error: `User ID ${userId} not found` },
        { status: 404 }
      )
    }
    return Response.json(
      { verified: false, error: 'Verification failed' + (stderr ? `: ${stderr.trim()}` : '') },
      { status: 500 }
    )
  }
}
