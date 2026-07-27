"use client";

import Head from "next/head";
import { useRouter } from "next/router";
import { useState, useEffect } from "react";
import { ConnectButton } from "../../components/ConnectButton";
import { useWalletStore } from "../../context/WalletContext";
import { managerService } from "../../lib/api";
import { shallow } from "../../lib/createStore";

type Step = "connect" | "register" | "submitted" | "status";

export default function ManagerOnboarding() {
  const router = useRouter();
  const { address } = useWalletStore((s) => ({ address: s.address }), shallow);
  const [step, setStep] = useState<Step>("connect");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [kycRef, setKycRef] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [managerRecord, setManagerRecord] = useState<Awaited<ReturnType<typeof managerService.register>> | null>(null);

  useEffect(() => {
    if (typeof window !== "undefined") {
      try {
        const savedDraft = sessionStorage.getItem("onboarding_draft");
        if (savedDraft) {
          const parsed = JSON.parse(savedDraft);
          if (parsed.name) setName(parsed.name);
          if (parsed.email) setEmail(parsed.email);
          if (parsed.kycRef) setKycRef(parsed.kycRef);
        }
      } catch {
        // ignore storage parse errors
      }
    }
  }, []);

  useEffect(() => {
    if (typeof window !== "undefined" && step === "register") {
      try {
        sessionStorage.setItem(
          "onboarding_draft",
          JSON.stringify({ name, email, kycRef })
        );
      } catch {
        // ignore storage errors
      }
    }
  }, [name, email, kycRef, step]);

  useEffect(() => {
    if (address) {
      checkExisting(address);
    }
  }, [address]);

  async function checkExisting(addr: string) {
    try {
      const status = await managerService.checkStatus(addr);
      if (status.status === "approved") {
        setManagerRecord({ id: status.id, stellar_address: addr, name: "", email: "", status: status.status, kyc_document_ref: "", notes: "", created_at: "", updated_at: "" });
        setStep("status");
      } else if (status.status === "pending") {
        setStep("submitted");
      } else if (status.status === "rejected") {
        setError("Your registration was rejected.");
        setStep("status");
      } else {
        setStep("register");
      }
    } catch {
      setStep("register");
    }
  }

  async function handleRegister(e: React.FormEvent) {
    e.preventDefault();
    if (!address) return;
    if (!name.trim()) {
      setError("Name is required");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const record = await managerService.register({
        stellar_address: address,
        name: name.trim(),
        email: email.trim(),
        kyc_document_ref: kycRef.trim(),
      });
      setManagerRecord(record);
      setStep("submitted");
      if (typeof window !== "undefined") {
        sessionStorage.removeItem("onboarding_draft");
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Registration failed";
      setError(msg);
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <Head>
        <title>Manager Onboarding | Perigee</title>
      </Head>
      <main className="min-h-screen bg-slate-950 text-slate-100">
        <header className="sticky top-0 z-50 border-b border-slate-800 bg-slate-950/90 backdrop-blur">
          <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">
            <div>
              <h1 className="text-2xl font-bold text-cyan-400">Perigee</h1>
              <p className="text-sm text-slate-400">Manager Onboarding</p>
            </div>
            <ConnectButton />
          </div>
        </header>

        <section className="mx-auto max-w-2xl px-4 py-12 sm:px-6 lg:px-8">
          <div className="rounded-2xl border border-slate-800 bg-slate-900/70 p-8">
            <h2 className="mb-6 text-2xl font-semibold text-cyan-300">
              {step === "connect" && "Connect Your Wallet"}
              {step === "register" && "Register as a Manager"}
              {step === "submitted" && "Registration Submitted"}
              {step === "status" && "Manager Status"}
            </h2>

            {step === "connect" && (
              <div className="space-y-4">
                <p className="text-slate-400">
                  Connect your Stellar wallet to begin manager registration.
                </p>
              </div>
            )}

            {step === "register" && (
              <form onSubmit={handleRegister} className="space-y-5">
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-300">Stellar Address</label>
                  <input
                    value={address || ""}
                    disabled
                    className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 font-mono text-sm text-slate-400"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-300">Full Name *</label>
                  <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Your name or business name"
                    className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                    required
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-300">Email</label>
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="contact@example.com"
                    className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-slate-300">KYC Document Reference</label>
                  <input
                    value={kycRef}
                    onChange={(e) => setKycRef(e.target.value)}
                    placeholder="Optional: ID/document number"
                    className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                  />
                </div>

                {error && (
                  <div className="rounded-lg border border-red-800 bg-red-950/50 px-4 py-3 text-sm text-red-400">
                    {error}
                  </div>
                )}

                <button
                  type="submit"
                  disabled={loading}
                  className="rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-semibold text-white hover:bg-cyan-500 disabled:opacity-50 transition-colors"
                >
                  {loading ? "Submitting..." : "Submit Registration"}
                </button>
              </form>
            )}

            {step === "submitted" && (
              <div className="space-y-4">
                <div className="rounded-lg border border-yellow-800 bg-yellow-950/50 px-4 py-3 text-sm text-yellow-400">
                  Your registration has been submitted and is pending approval.
                  You will be able to create vaults once an operator approves your account.
                </div>
                {managerRecord && (
                  <div className="space-y-2 text-sm text-slate-400">
                    <p><span className="text-slate-300">ID:</span> {managerRecord.id}</p>
                    <p><span className="text-slate-300">Status:</span> {managerRecord.status}</p>
                  </div>
                )}
              </div>
            )}

            {step === "status" && (
              <div className="space-y-4">
                {error ? (
                  <div className="rounded-lg border border-red-800 bg-red-950/50 px-4 py-3 text-sm text-red-400">
                    {error}
                  </div>
                ) : (
                  <div className="rounded-lg border border-green-800 bg-green-950/50 px-4 py-3 text-sm text-green-400">
                    Your manager account is approved and active. You can now create vaults.
                  </div>
                )}
              </div>
            )}

            <div className="mt-8 border-t border-slate-800 pt-4">
              <button
                onClick={() => router.push("/")}
                className="text-sm text-cyan-400 hover:text-cyan-300 transition-colors"
              >
                &larr; Back to Analyzer
              </button>
            </div>
          </div>
        </section>
      </main>
    </>
  );
}
