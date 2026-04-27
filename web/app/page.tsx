import path from 'path'
import fs from 'fs'
import InclusionChecker from '../_components/InclusionChecker'

interface Attestation {
  merkleRoot: string
  assetsCommitment: string
  totalLiabilities: bigint
  totalAssets: bigint
}

function decodePublicValues(hex: string): Attestation | null {
  const raw = hex.replace(/^0x/i, '')
  if (raw.length < 256) return null
  return {
    merkleRoot:       '0x' + raw.slice(0, 64),
    assetsCommitment: '0x' + raw.slice(64, 128),
    totalLiabilities: BigInt('0x' + raw.slice(128, 192)),
    totalAssets:      BigInt('0x' + raw.slice(192, 256)),
  }
}

export default function Home() {
  let attestation: Attestation | null = null
  try {
    const raw = JSON.parse(fs.readFileSync(path.join(process.cwd(), '..', 'proof.json'), 'utf-8'))
    attestation = decodePublicValues(raw.public_values)
  } catch { /* proof.json missing — render gracefully */ }

  const surplus = attestation ? attestation.totalAssets - attestation.totalLiabilities : null

  return (
    <main className="min-h-screen bg-gray-50">
      <div className="max-w-2xl mx-auto px-4 py-12">

        <div className="mb-10">
          <h1 className="text-3xl font-bold text-gray-900 tracking-tight">ZK Solvency Protocol</h1>
          <p className="mt-2 text-gray-500 text-sm">
            Cryptographic proof that total assets &ge; total liabilities, without revealing individual balances.
          </p>
        </div>

        <section className="mb-8">
          <h2 className="text-xs font-semibold uppercase tracking-widest text-gray-400 mb-3">
            Latest Attestation
          </h2>
          {attestation ? (
            <div className="bg-white border border-gray-200 rounded-2xl p-6 shadow-sm space-y-4">
              <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-4 text-sm">
                <div className="sm:col-span-2">
                  <dt className="text-gray-500 font-medium mb-0.5">Merkle Root</dt>
                  <dd className="font-mono text-gray-800 text-xs break-all">{attestation.merkleRoot}</dd>
                </div>
                <div className="sm:col-span-2">
                  <dt className="text-gray-500 font-medium mb-0.5">Assets Commitment</dt>
                  <dd className="font-mono text-gray-800 text-xs break-all">{attestation.assetsCommitment}</dd>
                </div>
                <div>
                  <dt className="text-gray-500 font-medium mb-0.5">Total Liabilities</dt>
                  <dd className="font-mono text-gray-800">{attestation.totalLiabilities.toLocaleString()}</dd>
                </div>
                <div>
                  <dt className="text-gray-500 font-medium mb-0.5">Total Assets</dt>
                  <dd className="font-mono text-gray-800">{attestation.totalAssets.toLocaleString()}</dd>
                </div>
                <div className="sm:col-span-2">
                  <dt className="text-gray-500 font-medium mb-0.5">Surplus</dt>
                  <dd className="font-mono font-semibold text-emerald-600">+{surplus!.toLocaleString()}</dd>
                </div>
              </dl>
              <div className="pt-2 border-t border-gray-100">
                <a
                  href="https://sepolia.etherscan.io/address/0x397A5f7f3dBd538f23DE225B51f532c34448dA9B"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-xs text-indigo-600 hover:text-indigo-800 font-medium"
                >
                  View SP1 Groth16 Verifier on Sepolia ↗
                </a>
                <p className="text-xs text-gray-400 mt-1">
                  SolvencyAttestation.sol deployment pending — showing SP1 verifier gateway address
                </p>
              </div>
            </div>
          ) : (
            <div className="bg-amber-50 border border-amber-200 rounded-2xl p-6 text-sm text-amber-800">
              <p className="font-semibold mb-1">No proof found</p>
              <p className="font-mono text-xs text-amber-700">
                SP1_PROVER=mock cargo run --manifest-path script/Cargo.toml
              </p>
            </div>
          )}
        </section>

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-widest text-gray-400 mb-3">
            Verify Your Inclusion
          </h2>
          <InclusionChecker merkleRoot={attestation?.merkleRoot ?? ''} />
          <p className="mt-3 text-xs text-gray-400">
            Enter any user ID from 0 to 99 to verify that their balance is committed in the Merkle root above.
          </p>
        </section>

      </div>
    </main>
  )
}
