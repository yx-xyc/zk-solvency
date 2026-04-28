// id and balance are typed as number to match JSON deserialization.
// Values must be within Number.isSafeInteger range (< 2^53); le64() uses
// BigInt internally so encoding is correct, but JS number precision loss
// occurs before that for values >= 2^53. A future migration to bigint
// end-to-end would remove this limitation.
export interface User {
  id: number
  balance: number
}

export interface MerkleProof {
  leafIndex: number
  leaf: Uint8Array
  siblings: Uint8Array[]
  pathBits: boolean[]
}

async function sha256(data: Uint8Array): Promise<Uint8Array> {
  // Cast via ArrayBuffer — our Uint8Arrays always own their buffer (freshly allocated, not subviews)
  return new Uint8Array(await crypto.subtle.digest('SHA-256', data.buffer as ArrayBuffer))
}

function le64(n: number | bigint): Uint8Array {
  const buf = new ArrayBuffer(8)
  new DataView(buf).setBigUint64(0, BigInt(n), true)
  return new Uint8Array(buf)
}

export async function leafHash(id: number | bigint, balance: number | bigint): Promise<Uint8Array> {
  const data = new Uint8Array(16)
  data.set(le64(id), 0)
  data.set(le64(balance), 8)
  return sha256(data)
}

export async function nodeHash(left: Uint8Array, right: Uint8Array): Promise<Uint8Array> {
  const data = new Uint8Array(64)
  data.set(left, 0)
  data.set(right, 32)
  return sha256(data)
}

function nextPow2(n: number): number {
  let p = 1
  while (p < n) p <<= 1
  return p
}

export async function buildTree(users: User[]): Promise<{ root: Uint8Array; layers: Uint8Array[][] }> {
  const rawLeaves = await Promise.all(users.map(u => leafHash(u.id, u.balance)))

  // Pad to next power of two by repeating the last leaf — mirrors crates/types/src/merkle.rs
  const target = nextPow2(rawLeaves.length)
  const last = rawLeaves[rawLeaves.length - 1]
  const padded = [...rawLeaves]
  while (padded.length < target) padded.push(last)

  const layers: Uint8Array[][] = [padded]
  let current = padded
  while (current.length > 1) {
    const next: Uint8Array[] = []
    for (let i = 0; i < current.length; i += 2) {
      next.push(await nodeHash(current[i], current[i + 1]))
    }
    layers.push(next)
    current = next
  }
  return { root: current[0], layers }
}

export async function generateProof(users: User[], index: number): Promise<MerkleProof> {
  const { layers } = await buildTree(users)
  const leaf = layers[0][index]
  const siblings: Uint8Array[] = []
  const pathBits: boolean[] = []
  let idx = index
  for (let level = 0; level < layers.length - 1; level++) {
    const isLeft = idx % 2 === 0
    siblings.push(layers[level][isLeft ? idx + 1 : idx - 1])
    pathBits.push(isLeft)
    idx = Math.floor(idx / 2)
  }
  return { leafIndex: index, leaf, siblings, pathBits }
}

export async function verifyProof(root: Uint8Array, proof: MerkleProof): Promise<boolean> {
  let current = proof.leaf
  for (let i = 0; i < proof.siblings.length; i++) {
    current = proof.pathBits[i]
      ? await nodeHash(current, proof.siblings[i])
      : await nodeHash(proof.siblings[i], current)
  }
  return toHex(current) === toHex(root)
}

export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('')
}
