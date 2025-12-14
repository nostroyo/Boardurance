/**
 * RaceAPIService - Service layer for Race API endpoints
 * Handles all communication with the backend race API
 */

import type {
  CarData,
  PerformancePreview,
  TurnPhase,
  LocalView,
  BoostAvailability,
  LapHistory,
  SubmitActionRequest,
  SubmitActionResponse,
} from '../types/race-api';

/**
 * RaceAPIService class for interacting with backend race endpoints
 */
export class RaceAPIService {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:3000/api/v1') {
    this.baseUrl = baseUrl;
  }

  /**
   * Get default fetch options with authentication
   */
  private getAuthenticatedFetchOptions(options: RequestInit = {}): RequestInit {
    return {
      ...options,
      credentials: 'include', // Include cookies for authentication
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    };
  }

  /**
   * Handle API response and parse JSON with enhanced error handling
   */
  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      let errorMessage = `Request failed with status ${response.status}`;

      // Map common HTTP status codes to user-friendly messages
      switch (response.status) {
        case 400:
          errorMessage = 'Invalid request. Please check your input.';
          break;
        case 401:
          errorMessage = 'Authentication required. Please log in.';
          break;
        case 403:
          errorMessage = 'Access denied. You do not have permission for this action.';
          break;
        case 404:
          errorMessage = 'Resource not found. The race or player may no longer exist.';
          break;
        case 409:
          errorMessage = 'Conflict with current race state. Please refresh and try again.';
          break;
        case 429:
          errorMessage = 'Too many requests. Please wait a moment and try again.';
          break;
        case 500:
          errorMessage = 'Server error. Please try again later.';
          break;
        case 502:
        case 503:
        case 504:
          errorMessage = 'Server temporarily unavailable. Please try again.';
          break;
      }

      try {
        const errorData = await response.json();
        // Use backend error message if available, otherwise use our mapped message
        errorMessage = errorData.message || errorData.error || errorMessage;
      } catch {
        // If JSON parsing fails, use our mapped error message
      }

      const error = new Error(errorMessage);
      // Add status code to error for categorization
      (error as any).status = response.status;
      throw error;
    }

    try {
      return await response.json();
    } catch (parseError) {
      throw new Error('Invalid response format from server');
    }
  }

  /**
   * Enhanced fetch with network error handling, timeout, and cancellation support
   * Requirements: 12.3
   */
  private async fetchWithErrorHandling(
    url: string,
    options: RequestInit = {},
    timeout: number = 10000,
    externalAbortSignal?: AbortSignal,
  ): Promise<Response> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    // Combine internal abort signal with external one if provided
    const combinedSignal = externalAbortSignal
      ? this.combineAbortSignals([controller.signal, externalAbortSignal])
      : controller.signal;

    try {
      const response = await fetch(url, {
        ...options,
        signal: combinedSignal,
      });

      clearTimeout(timeoutId);
      return response;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error) {
        // Handle different types of network errors
        if (error.name === 'AbortError') {
          throw new Error('Request timed out. Please check your connection and try again.');
        }

        if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
          throw new Error('Network error. Please check your internet connection.');
        }

        if (
          error.message.includes('ERR_NETWORK') ||
          error.message.includes('ERR_INTERNET_DISCONNECTED')
        ) {
          throw new Error('No internet connection. Please check your network.');
        }
      }

      // Re-throw original error if we can't categorize it
      throw error;
    }
  }

  /**
   * Combine multiple AbortSignals into one
   */
  private combineAbortSignals(signals: AbortSignal[]): AbortSignal {
    const controller = new AbortController();

    for (const signal of signals) {
      if (signal.aborted) {
        controller.abort();
        break;
      }

      signal.addEventListener('abort', () => controller.abort(), { once: true });
    }

    return controller.signal;
  }

  /**
   * GET /api/v1/races/{race_uuid}/players/{player_uuid}/car-data
   * Fetch player's car, pilot, engine, and body information
   */
  async getCarData(raceUuid: string, playerUuid: string): Promise<CarData> {
    const url = `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/car-data`;

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({ method: 'GET' }),
    );

    return this.handleResponse<CarData>(response);
  }

  /**
   * GET /api/v1/races/{race_uuid}/players/{player_uuid}/performance-preview
   * Fetch performance predictions for all boost options
   */
  async getPerformancePreview(raceUuid: string, playerUuid: string): Promise<PerformancePreview> {
    const url = `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/performance-preview`;

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({ method: 'GET' }),
    );

    return this.handleResponse<PerformancePreview>(response);
  }

  /**
   * GET /api/v1/races/{race_uuid}/turn-phase
   * Fetch current turn phase and state
   */
  async getTurnPhase(raceUuid: string): Promise<TurnPhase> {
    const url = `${this.baseUrl}/races/${raceUuid}/turn-phase`;

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({ method: 'GET' }),
    );

    return this.handleResponse<TurnPhase>(response);
  }

  /**
   * GET /api/v1/races/{race_uuid}/players/{player_uuid}/local-view
   * Fetch visible sectors (current sector ±2) with participant positions
   */
  async getLocalView(raceUuid: string, playerUuid: string): Promise<LocalView> {
    const url = `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/local-view`;

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({ method: 'GET' }),
    );

    return this.handleResponse<LocalView>(response);
  }

  /**
   * GET /api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability
   * Fetch available boost cards and hand state
   */
  async getBoostAvailability(raceUuid: string, playerUuid: string): Promise<BoostAvailability> {
    const url = `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/boost-availability`;

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({ method: 'GET' }),
    );

    return this.handleResponse<BoostAvailability>(response);
  }

  /**
   * GET /api/v1/races/{race_uuid}/players/{player_uuid}/lap-history
   * Fetch lap-by-lap performance history
   */
  async getLapHistory(raceUuid: string, playerUuid: string): Promise<LapHistory> {
    const url = `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/lap-history`;

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({ method: 'GET' }),
    );

    return this.handleResponse<LapHistory>(response);
  }

  /**
   * POST /api/v1/races/{race_uuid}/submit-action
   * Submit boost selection for the current turn
   */
  async submitTurnAction(
    raceUuid: string,
    playerUuid: string,
    boostValue: number,
  ): Promise<SubmitActionResponse> {
    const url = `${this.baseUrl}/races/${raceUuid}/submit-action`;

    const requestBody: SubmitActionRequest = {
      player_uuid: playerUuid,
      boost_value: boostValue,
    };

    const response = await this.fetchWithErrorHandling(
      url,
      this.getAuthenticatedFetchOptions({
        method: 'POST',
        body: JSON.stringify(requestBody),
      }),
      15000, // Longer timeout for submission
    );

    return this.handleResponse<SubmitActionResponse>(response);
  }

  /**
   * Batch multiple API calls for better performance
   * Requirements: 12.3
   */
  async batchRaceData(
    raceUuid: string,
    playerUuid: string,
    requests: {
      includeLocalView?: boolean;
      includeBoostAvailability?: boolean;
      includeLapHistory?: boolean;
      includePerformancePreview?: boolean;
    } = {},
  ): Promise<{
    localView?: LocalView;
    boostAvailability?: BoostAvailability;
    lapHistory?: LapHistory;
    performancePreview?: PerformancePreview;
  }> {
    const promises: Promise<any>[] = [];
    const keys: string[] = [];

    if (requests.includeLocalView) {
      promises.push(this.getLocalView(raceUuid, playerUuid));
      keys.push('localView');
    }

    if (requests.includeBoostAvailability) {
      promises.push(this.getBoostAvailability(raceUuid, playerUuid));
      keys.push('boostAvailability');
    }

    if (requests.includeLapHistory) {
      promises.push(this.getLapHistory(raceUuid, playerUuid));
      keys.push('lapHistory');
    }

    if (requests.includePerformancePreview) {
      promises.push(this.getPerformancePreview(raceUuid, playerUuid));
      keys.push('performancePreview');
    }

    try {
      const results = await Promise.all(promises);

      const batchResult: any = {};
      results.forEach((result, index) => {
        batchResult[keys[index]] = result;
      });

      return batchResult;
    } catch (error) {
      console.error('[RaceAPIService] Batch request failed:', error);
      throw error;
    }
  }
}

/**
 * Default instance of RaceAPIService
 * Can be imported and used directly: import { raceAPIService } from './services/raceAPI'
 */
export const raceAPIService = new RaceAPIService();
