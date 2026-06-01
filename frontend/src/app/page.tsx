'use client';

import { useState, useCallback } from 'react';
import { WalletConnect } from '@/components/WalletConnect';
import { RadarDashboard } from '@/components/RadarDashboard';
import { TransactionLog } from '@/components/TransactionLog';

export default function Home() {
  const [rtl, setRtl] = useState(false);

  const toggleRtl = useCallback(() => {
    const next = !rtl;
    setRtl(next);
    document.documentElement.dir = next ? 'rtl' : 'ltr';
  }, [rtl]);

  return (
    <div className="min-h-screen bg-[#0f0f13]" dir={rtl ? 'rtl' : 'ltr'}>
      {/* Background Effects */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute -top-40 -right-40 w-96 h-96 bg-indigo-500/10 rounded-full blur-3xl" />
        <div className="absolute -bottom-40 -left-40 w-96 h-96 bg-purple-500/10 rounded-full blur-3xl" />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-indigo-600/5 rounded-full blur-3xl" />
      </div>

      {/* Content */}
      <div className="relative z-10">
        {/* Top Navigation */}
        <header className="sticky top-0 z-50 glass border-b border-white/5">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex items-center justify-between h-16">
              {/* Logo */}
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center">
                  <svg className="w-4 h-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M13 10V3L4 14h7v7l9-11h-7z"
                    />
                  </svg>
                </div>
                <div>
                  <h1 className="text-sm font-bold text-white tracking-tight">
                    {rtl ? '\u0635\u0641\u0642\u0629 \u0631\u0623\u0633 \u0645\u0627\u0644 \u0635\u0641\u0631' : 'Zero-Cap Arbitrage'}
                  </h1>
                  <p className="text-[10px] text-gray-500 -mt-0.5">
                    {rtl ? '0 \u0631\u0623\u0633 \u0645\u0627\u0644 \u2022 0 \u063a\u0627\u0632 \u0645\u0642\u062f\u0645 \u2022 \u0632\u0645\u0646 \u0627\u0633\u062a\u062c\u0627\u0628\u0629 \u0641\u0627\u0626\u0642 \u0627\u0644\u0633\u0631\u0639\u0629' : '0 Capital \u2022 0 Upfront Gas \u2022 Ultra-Low Latency'}
                  </p>
                </div>
              </div>

              {/* Right section */}
              <div className="flex items-center gap-4">
                <div className="hidden sm:flex items-center gap-3 text-xs text-gray-500">
                  <span className="flex items-center gap-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                    {rtl ? '6 \u0633\u0644\u0627\u0633\u0644' : '6 Chains'}
                  </span>
                  <span className="flex items-center gap-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
                    {rtl ? '50+ \u062a\u0628\u0627\u062f\u0644' : '50+ DEXes'}
                  </span>
                </div>
                <button
                  onClick={toggleRtl}
                  className="px-2 py-1 text-xs font-medium text-gray-400 hover:text-white 
                             border border-white/10 rounded-lg hover:border-white/20 transition-colors"
                  title="Toggle Arabic/RTL"
                >
                  {rtl ? 'EN' : '\u0639\u0631\u0628\u064a'}
                </button>
                <WalletConnect />
              </div>
            </div>
          </div>
        </header>

        {/* Main Dashboard */}
        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="mb-8">
            <h2 className="text-2xl font-bold text-white">
              {rtl ? '\u0631\u0627\u062f\u0627\u0631 \u0627\u0644\u0645\u0631\u0627\u0628\u062d\u0629 \u0641\u064a \u0627\u0644\u0648\u0642\u062a \u0627\u0644\u062d\u0642\u064a\u0642\u064a' : 'Real-Time '}
              <span className="gradient-text">
                {rtl ? '\u0631\u0627\u062f\u0627\u0631 \u0627\u0644\u0645\u0631\u0627\u0628\u062d\u0629' : 'Arbitrage Radar'}
              </span>
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              {rtl
                ? '\u0627\u0645\u0633\u062d 50+ \u062a\u0628\u0627\u062f\u0644 \u0639\u0628\u0631 6 \u0633\u0644\u0627\u0633\u0644. \u0646\u0641\u0630 \u0628\u062f\u0648\u0646 \u0631\u0623\u0633 \u0645\u0627\u0644 \u0639\u0628\u0631 \u0627\u0644\u0642\u0631\u0648\u0636 \u0627\u0644\u0648\u0645\u064a\u0636\u0629. \u0627\u062f\u0641\u0639 0 \u063a\u0627\u0632 \u0645\u0642\u062f\u0645 \u0639\u0628\u0631 Flashbots \u0623\u0648 ERC-4337.'
                : 'Scan 50+ DEXes across 6 chains. Execute with 0 capital via flash loans. Pay 0 upfront gas via Flashbots or ERC-4337.'
              }
            </p>
          </div>

          <div className="space-y-6">
            <RadarDashboard />
            <TransactionLog />
          </div>
        </main>

        {/* Footer */}
        <footer className="border-t border-white/5 py-6 mt-12">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex items-center justify-between text-xs text-gray-600">
              <p>Zero-Cap Arbitrage v0.1.0 &bull; {rtl ? '\u0644\u064a\u0633\u062a \u0646\u0635\u064a\u062d\u0629 \u0645\u0627\u0644\u064a\u0629' : 'Not financial advice'}</p>
              <div className="flex items-center gap-4">
                <a
                  href="https://paraswap.io"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-gray-400 transition-colors"
                >
                  Powered by Velora (ex-ParaSwap)
                </a>
                <span>|</span>
                <a
                  href="https://flashbots.net"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-gray-400 transition-colors"
                >
                  Flashbots
                </a>
                <span>|</span>
                <a
                  href="https://pimlico.io"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-gray-400 transition-colors"
                >
                  Pimlico
                </a>
              </div>
            </div>
          </div>
        </footer>
      </div>
    </div>
  );
}
