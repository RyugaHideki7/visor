"use client";

import HfsqlConfigTab from "../HfsqlConfigTab";

export default function AteisConnexionPage() {
  return (
    <div className="p-8 flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>
          Data exchange · Ateis · Connexion
        </h1>
        <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
          Configuration de la connexion ODBC vers HFSQL pour tous les exports ATEIS.
        </p>
      </div>

      <HfsqlConfigTab />
    </div>
  );
}
