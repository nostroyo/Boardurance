import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TrackDisplayRedesign } from './TrackDisplayRedesign';
import { BoostControlPanel } from './BoostControlPanel';
import { SectorGrid } from './SectorGrid';
import type { LocalView, BoostAvailability } from '../../types/race-api';
import type { TurnPhase } from '../../types/race';

// Mock data for testing
const mockLocalView: LocalView = {
  center_sector: 1,
  visible_sectors: [
    {
      id: 0,
      name: 'Start Line',
      min_value: 1,
      max_value: 3,
      slot_capacity: 5,
      current_occupancy: 2,
    },
    {
      id: 1,
      name: 'First Turn',
      min_value: 2,
      max_value: 4,
      slot_capacity: 5,
      current_occupancy: 1,
    },
    {
      id: 2,
      name: 'Back Straight',
      min_value: 3,
      max_value: 5,
      slot_capacity: 5,
      current_occupancy: 0,
    },
  ],
  visible_participants: [
    {
      player_uuid: 'player-1',
      player_name: 'Test Player',
      car_name: 'Test Car',
      current_sector: 1,
      position_in_sector: 1,
      current_lap: 1,
      total_value: 10,
    },
    {
      player_uuid: 'player-2',
      player_name: 'Other Player',
      car_name: 'Other Car',
      current_sector: 0,
      position_in_sector: 1,
      current_lap: 1,
      total_value: 8,
    },
  ],
};

// Mock window.matchMedia for responsive testing
const mockMatchMedia = (query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
});

// Helper to simulate different screen sizes
const setScreenSize = (width: number) => {
  Object.defineProperty(window, 'innerWidth', {
    writable: true,
    configurable: true,
    value: width,
  });

  // Mock matchMedia for different breakpoints
  window.matchMedia = vi.fn().mockImplementation((query) => {
    const breakpoints = {
      '(min-width: 1024px)': width >= 1024, // lg
      '(min-width: 768px)': width >= 768,   // md
      '(min-width: 640px)': width >= 640,   // sm
      '(min-width: 475px)': width >= 475,   // xs
    };
    
    return {
      ...mockMatchMedia(query),
      matches: breakpoints[query as keyof typeof breakpoints] || false,
    };
  });

  // Trigger resize event
  window.dispatchEvent(new Event('resize'));
};

describe('Responsive Layout Tests', () => {
  beforeEach(() => {
    // Reset to default desktop size
    setScreenSize(1024);
  });

  describe('TrackDisplayRedesign Responsive Behavior', () => {
    it('should render with desktop layout on large screens', () => {
      setScreenSize(1024);
      
      render(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      // Check that the component renders
      expect(screen.getByText('Track View')).toBeInTheDocument();
      
      // Check for desktop-specific classes (these would be applied by Tailwind)
      const header = screen.getByText('Track View').closest('div');
      expect(header).toHaveClass('lg:px-6'); // Desktop padding
    });

    it('should render with tablet layout on medium screens', () => {
      setScreenSize(768);
      
      render(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      // Should still show most desktop features but with adjusted spacing
      expect(screen.getByText('Track View')).toBeInTheDocument();
      expect(screen.getByText('Sector 1')).toBeInTheDocument();
    });

    it('should render with mobile layout on small screens', () => {
      setScreenSize(375);
      
      render(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      // Mobile should have compact layout
      expect(screen.getByText('Track View')).toBeInTheDocument();
      const header = screen.getByText('Track View').closest('div');
      expect(header).toHaveClass('px-3'); // Mobile padding
    });

    it('should adapt sector grid height for different screen sizes', () => {
      // Test mobile height
      setScreenSize(375);
      const { rerender } = render(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      let scrollContainer = document.querySelector('.h-64');
      expect(scrollContainer).toBeInTheDocument();

      // Test desktop height
      setScreenSize(1024);
      rerender(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      scrollContainer = document.querySelector('.lg\\:h-\\[28rem\\]');
      expect(scrollContainer).toBeInTheDocument();
    });
  });

  describe('BoostControlPanel Responsive Behavior', () => {
    const mockProps = {
      selectedBoost: null,
      availableBoosts: [0, 1, 2, 3, 4, 5],
      onBoostSelect: vi.fn(),
      onValidateTurn: vi.fn(),
      isSubmitting: false,
      hasSubmitted: false,
      turnPhase: 'WaitingForPlayers' as TurnPhase,
    };

    it('should render 6-column grid on desktop', () => {
      setScreenSize(1024);
      
      render(<BoostControlPanel {...mockProps} />);

      const buttonGrid = document.querySelector('.sm\\:grid-cols-6');
      expect(buttonGrid).toBeInTheDocument();
      
      // All boost buttons should be visible
      for (let i = 0; i <= 5; i++) {
        expect(screen.getByRole('button', { name: `Select boost value ${i}` })).toBeInTheDocument();
      }
    });

    it('should render 3-column grid on mobile', () => {
      setScreenSize(375);
      
      render(<BoostControlPanel {...mockProps} />);

      const buttonGrid = document.querySelector('.grid-cols-3');
      expect(buttonGrid).toBeInTheDocument();
      
      // All boost buttons should still be accessible
      for (let i = 0; i <= 5; i++) {
        expect(screen.getByRole('button', { name: `Select boost value ${i}` })).toBeInTheDocument();
      }
    });

    it('should have touch-friendly button sizes on mobile', () => {
      setScreenSize(375);
      
      render(<BoostControlPanel {...mockProps} />);

      const buttons = screen.getAllByRole('button', { name: /Select boost value/ });
      buttons.forEach(button => {
        expect(button).toHaveClass('min-h-[48px]'); // Touch-friendly minimum height
        expect(button).toHaveClass('touch-manipulation'); // Touch optimization
      });
    });

    it('should stack confirmation dialog buttons on mobile', () => {
      setScreenSize(375);
      
      render(<BoostControlPanel {...mockProps} selectedBoost={3} />);

      // Click validate to show confirmation
      const validateButton = screen.getByRole('button', { name: /Validate turn/ });
      validateButton.click();

      // Check for mobile stacking - look for the actual grid class used
      const buttonContainer = document.querySelector('.sm\\:grid-cols-2');
      expect(buttonContainer).toBeInTheDocument();
    });
  });

  describe('SectorGrid Responsive Behavior', () => {
    const mockSector = mockLocalView.visible_sectors[0];
    const mockParticipants = mockLocalView.visible_participants;

    it('should render 2-column info grid on desktop', () => {
      setScreenSize(1024);
      
      render(
        <SectorGrid
          sector={mockSector}
          participants={mockParticipants}
          isPlayerSector={false}
          playerUuid="player-1"
        />
      );

      const infoGrid = document.querySelector('.sm\\:grid-cols-2');
      expect(infoGrid).toBeInTheDocument();
    });

    it('should render single-column info grid on mobile', () => {
      setScreenSize(375);
      
      render(
        <SectorGrid
          sector={mockSector}
          participants={mockParticipants}
          isPlayerSector={false}
          playerUuid="player-1"
        />
      );

      const infoGrid = document.querySelector('.grid-cols-1');
      expect(infoGrid).toBeInTheDocument();
    });

    it('should wrap position slots on mobile', () => {
      setScreenSize(375);
      
      render(
        <SectorGrid
          sector={mockSector}
          participants={mockParticipants}
          isPlayerSector={false}
          playerUuid="player-1"
        />
      );

      const slotContainer = document.querySelector('.flex-wrap');
      expect(slotContainer).toBeInTheDocument();
    });
  });

  describe('Cross-Component Layout Integration', () => {
    it('should maintain proper spacing across all breakpoints', () => {
      const breakpoints = [375, 640, 768, 1024, 1280];
      
      breakpoints.forEach(width => {
        setScreenSize(width);
        
        const { rerender } = render(
          <div className="space-y-4 sm:space-y-6">
            <TrackDisplayRedesign
              localView={mockLocalView}
              playerUuid="player-1"
            />
            <BoostControlPanel
              selectedBoost={null}
              availableBoosts={[0, 1, 2, 3, 4, 5]}
              onBoostSelect={vi.fn()}
              onValidateTurn={vi.fn()}
              isSubmitting={false}
              hasSubmitted={false}
              turnPhase="WaitingForPlayers"
            />
          </div>
        );

        // Components should render without layout issues
        expect(screen.getByText('Track View')).toBeInTheDocument();
        expect(screen.getByText('Boost Control')).toBeInTheDocument();
        
        rerender(<div />); // Clean up for next iteration
      });
    });

    it('should handle text truncation on small screens', () => {
      setScreenSize(320); // Very small screen
      
      render(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      // Text should be truncated appropriately
      const trackTitle = screen.getByText('Track View');
      expect(trackTitle).toHaveClass('truncate');
    });
  });

  describe('Accessibility on Different Screen Sizes', () => {
    it('should maintain touch targets of at least 44px on mobile', () => {
      setScreenSize(375);
      
      render(
        <BoostControlPanel
          selectedBoost={null}
          availableBoosts={[0, 1, 2, 3, 4, 5]}
          onBoostSelect={vi.fn()}
          onValidateTurn={vi.fn()}
          isSubmitting={false}
          hasSubmitted={false}
          turnPhase="WaitingForPlayers"
        />
      );

      const buttons = screen.getAllByRole('button');
      buttons.forEach(button => {
        // Should have minimum touch target size
        expect(button).toHaveClass('min-h-[48px]');
      });
    });

    it('should provide appropriate text sizes for readability', () => {
      setScreenSize(375);
      
      render(
        <TrackDisplayRedesign
          localView={mockLocalView}
          playerUuid="player-1"
        />
      );

      // Text should be readable on mobile - check for responsive text classes
      const statusText = screen.getByText(/Player in sector/);
      expect(statusText).toHaveClass('text-xs');
    });
  });
});