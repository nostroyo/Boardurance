import { describe, it, expect, vi, afterEach } from 'vitest';
import { RaceAPIService } from './raceAPI';

describe('RaceAPIService.createSoloRace', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('POSTs the player UUID to /races/solo and returns the created race', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 201,
      json: async () => ({
        race: { uuid: 'solo-race-123', status: 'InProgress' },
        message: 'Solo race created and started',
      }),
    });
    vi.stubGlobal('fetch', fetchMock);

    const api = new RaceAPIService('/api/v1');
    const result = await api.createSoloRace('player-abc');

    expect(result.race.uuid).toBe('solo-race-123');

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/races/solo');
    expect(options.method).toBe('POST');
    expect(JSON.parse(options.body)).toEqual({ player_uuid: 'player-abc' });
  });

  it('throws a friendly error when the backend rejects the request', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      json: async () => ({ error: 'no complete car' }),
    });
    vi.stubGlobal('fetch', fetchMock);

    const api = new RaceAPIService('/api/v1');
    await expect(api.createSoloRace('player-abc')).rejects.toThrow();
  });
});
