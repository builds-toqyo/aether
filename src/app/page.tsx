"use client"
import { invoke } from "@tauri-apps/api/core";

export default function Home() {
  const handleClick = async () => {
    try {
      const message = await invoke("greet", { name: "World" });
      alert(message);
    } catch (error) {
      console.error("Error invoking Tauri command:", error);
    }
  };

  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <h1 className="text-4xl font-bold mb-8 text-green">Hello World from Aether!</h1>
      <p className="text-xl mb-8">Welcome to your Tauri + Next.js application</p>
      
      <button 
        onClick={handleClick}
        className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
      >
        Greet from Rust
      </button>
    </main>
  );
}
