import path from 'path'
import fs from 'fs'
import { buildTree, generateProof, leafHash, toHex, type User } from '../../../_lib/merkle'

const USERS_FILE = path.join(process.cwd(), '..', 'data', 'users.json')

export async function POST(req: Request) {
  let userId: unknown
  try {
    const body = await req.json()
    userId = body.userId
  } catch {
    return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
  }

  if (typeof userId !== 'number' || !Number.isSafeInteger(userId) || userId < 0) {
    return Response.json(
      { error: 'userId must be a non-negative integer within the safe integer range (< 2^53)' },
      { status: 400 }
    )
  }

  let users: User[]
  try {
    users = JSON.parse(fs.readFileSync(USERS_FILE, 'utf-8'))
  } catch {
    return Response.json(
      { error: 'data/users.json not found — run: cargo run -p data-gen' },
      { status: 500 }
    )
  }

  const index = users.findIndex(u => u.id === userId)
  if (index === -1) {
    return Response.json({ error: `User ID ${userId} not found` }, { status: 404 })
  }

  const user = users[index]

  // Build the tree and generate the proof server-side.
  // Only the requesting user's {balance, siblings, pathBits} is returned —
  // no other users' balances are exposed.
  const proof = await generateProof(users, index)
  const leaf = await leafHash(user.id, user.balance)
  const { root } = await buildTree(users)

  return Response.json({
    balance:    user.balance,
    leafHash:   '0x' + toHex(leaf),
    merkleRoot: '0x' + toHex(root),
    siblings:   proof.siblings.map(s => '0x' + toHex(s)),
    pathBits:   proof.pathBits,
    proofDepth: proof.siblings.length,
  })
}
