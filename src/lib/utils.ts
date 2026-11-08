/**
 * Format seconds into timecode string (HH:MM:SS:FF or MM:SS:FF)
 * @param seconds - time in seconds
 * @param fps - frames per second (default: 30)
 * @returns formatted timecode string
 */
export function formatTime(seconds: number, fps: number = 30): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  const frames = Math.floor((seconds % 1) * fps);

  const pad = (n: number) => n.toString().padStart(2, "0");

  if (hours > 0) {
    return `${pad(hours)}:${pad(minutes)}:${pad(secs)}:${pad(frames)}`;
  }
  return `${pad(minutes)}:${pad(secs)}:${pad(frames)}`;
}

/**
 * Format bytes into human-readable string (KB, MB, GB)
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

/**
 * Clamp a value between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}
