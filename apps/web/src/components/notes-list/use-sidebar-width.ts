import { useState } from "react";

export const SIDEBAR_WIDTH_STORAGE_KEY = "nnn-sidebar-width";
export const MIN_SIDEBAR_WIDTH = 220;
export const MAX_SIDEBAR_WIDTH = 420;
const DEFAULT_SIDEBAR_WIDTH = 248;

export function clampSidebarWidth(width: number) {
  return Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, width));
}

function getStoredSidebarWidth() {
  const storedWidth = Number(localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY));
  return Number.isFinite(storedWidth)
    ? clampSidebarWidth(storedWidth)
    : DEFAULT_SIDEBAR_WIDTH;
}

export function useStoredSidebarWidth() {
  return useState(getStoredSidebarWidth);
}
