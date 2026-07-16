// Central API endpoint configuration.
// Set VITE_API_BASE_URL at build time to target a deployed backend;
// defaults to the local dev server.
export const API_BASE_URL: string = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

export const API_V1_URL = `${API_BASE_URL}/api/v1`;
