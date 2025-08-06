import { useState } from "react";

export function useTimeline() {
  const [timeline, setTimeline] = useState([]);
  return { timeline, setTimeline };
}
