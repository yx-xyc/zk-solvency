'use client'
import { useState } from 'react'

type Result = {
  verified: boolean
  userId: number
  balance?: number
  leafHash?: string
  merkleRoot?: string
  proofDepth?: number
  error?: string
}

export default function InclusionChecker() {
  const [userId, setUserId] = useState('')
  const [result, setResult] = useState<Result | null>(null)
  const [loading, setLoading] = useState(false)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!userId) return
    setLoading(true)
    setResult(null)
    try {
      const res = await fetch('/api/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ userId: Number(userId) }),
      })
      const data: Result = await res.json()
      setResult(data)
    } catch {
      setResult({ verified: false, userId: Number(userId), error: 'Network error' })
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="bg-white border border-gray-200 rounded-2xl p-6 shadow-sm">
      <form onSubmit={handleSubmit} className="flex gap-3 items-end">
        <div className="flex-1">
          <label htmlFor="userId" className="block text-sm font-medium text-gray-700 mb-1">
            Your User ID
          </label>
          <input
            id="userId"
            type="number"
            min={0}
            step={1}
            value={userId}
            onChange={e => setUserId(e.target.value)}
            placeholder="0 – 99"
            required
            disabled={loading}
            className="w-full rounded-lg border border-gray-300 px-4 py-2.5 text-sm shadow-sm
                       focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500
                       disabled:bg-gray-50"
          />
        </div>
        <button
          type="submit"
          disabled={loading || !userId}
          className="px-5 py-2.5 rounded-lg bg-indigo-600 text-white text-sm font-semibold shadow-sm
                     hover:bg-indigo-700 active:bg-indigo-800 disabled:opacity-50 disabled:cursor-not-allowed
                     transition-colors"
        >
          {loading ? 'Checking…' : 'Verify Inclusion'}
        </button>
      </form>

      {result && (
        <div className={`mt-5 rounded-xl border p-4 ${
          result.verified ? 'bg-emerald-50 border-emerald-200' : 'bg-red-50 border-red-200'
        }`}>
          {result.verified ? (
            <>
              <p className="flex items-center gap-2 font-semibold text-emerald-700 mb-3">
                <span>✓</span>
                User {result.userId} is included in the committed Merkle root
              </p>
              <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2 text-sm">
                <div>
                  <dt className="text-gray-500 font-medium">Balance</dt>
                  <dd className="font-mono text-gray-800">{result.balance?.toLocaleString()}</dd>
                </div>
                <div>
                  <dt className="text-gray-500 font-medium">Proof Depth</dt>
                  <dd className="font-mono text-gray-800">{result.proofDepth} levels</dd>
                </div>
                <div className="sm:col-span-2">
                  <dt className="text-gray-500 font-medium">Leaf Hash</dt>
                  <dd className="font-mono text-gray-800 break-all text-xs">{result.leafHash}</dd>
                </div>
                <div className="sm:col-span-2">
                  <dt className="text-gray-500 font-medium">Merkle Root (recomputed)</dt>
                  <dd className="font-mono text-gray-800 break-all text-xs">{result.merkleRoot}</dd>
                </div>
              </dl>
            </>
          ) : (
            <p className="flex items-center gap-2 font-semibold text-red-700">
              <span>✗</span>
              {result.error ?? 'User not found or verification failed'}
            </p>
          )}
        </div>
      )}
    </div>
  )
}
