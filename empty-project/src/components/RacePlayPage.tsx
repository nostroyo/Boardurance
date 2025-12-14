/**
 * RacePlayPage - Wrapper component for the race play route
 *
 * This component:
 * - Extracts raceUuid from URL parameters
 * - Gets playerUuid from authentication context
 * - Handles navigation on race completion and errors
 * - Provides confirmation dialog for leaving active race
 *
 * Requirements: 1.1, 7.5, 9.4
 */

import { useParams, useNavigate } from 'react-router-dom';
import { useAuthContext } from '../contexts/AuthContext';
import { useEffect, useState } from 'react';
import RaceContainer from './player-game-interface/RaceContainer';

/**
 * Confirmation dialog component for leaving active race
 */
interface LeaveRaceDialogProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

function LeaveRaceDialog({ isOpen, onConfirm, onCancel }: LeaveRaceDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 border border-gray-600 rounded-lg p-6 max-w-md mx-4">
        <h2 className="text-white text-xl font-bold mb-4">Leave Race?</h2>
        <p className="text-gray-300 mb-6">
          Are you sure you want to leave this race? Your progress will be lost and you won't be able
          to rejoin.
        </p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded transition-colors"
          >
            Stay in Race
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded transition-colors"
          >
            Leave Race
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * RacePlayPage component
 */
export default function RacePlayPage() {
  const { raceUuid } = useParams<{ raceUuid: string }>();
  const { user, isAuthenticated } = useAuthContext();
  const navigate = useNavigate();
  const [showLeaveDialog, setShowLeaveDialog] = useState(false);

  // Redirect to login if not authenticated
  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login', { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // Validate required parameters
  if (!raceUuid) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="bg-red-900 border border-red-700 rounded-lg p-6 max-w-md">
          <h2 className="text-white text-xl font-bold mb-2">Invalid Race</h2>
          <p className="text-red-200 mb-4">
            Race UUID is missing from the URL. Please check the link and try again.
          </p>
          <button
            onClick={() => navigate('/game')}
            className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
          >
            Return to Lobby
          </button>
        </div>
      </div>
    );
  }

  if (!user) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <div className="text-xl text-gray-300">Loading user data...</div>
        </div>
      </div>
    );
  }

  /**
   * Handle race completion - navigate to race lobby
   * Requirements: 7.5
   */
  const handleRaceComplete = (finalPosition: number) => {
    console.log(`[RacePlayPage] Race completed with position: ${finalPosition}`);

    // Navigate to race lobby with completion message
    navigate('/game', {
      state: {
        message: `Race completed! You finished in position ${finalPosition}.`,
        type: 'success',
      },
    });
  };

  /**
   * Handle race errors - navigate based on error type
   * Requirements: 9.4
   */
  const handleRaceError = (error: Error) => {
    console.error('[RacePlayPage] Race error:', error);

    // Determine navigation based on error type
    if (error.message.includes('race not found') || error.message.includes('404')) {
      // Race not found - return to lobby with error message
      navigate('/game', {
        state: {
          message: 'Race not found. It may have been cancelled or completed.',
          type: 'error',
        },
      });
    } else if (error.message.includes('player not in race') || error.message.includes('403')) {
      // Player not authorized for this race
      navigate('/game', {
        state: {
          message: 'You are not authorized to join this race.',
          type: 'error',
        },
      });
    } else {
      // Generic error - stay on page but show error
      console.error('[RacePlayPage] Unhandled race error:', error);
    }
  };

  /**
   * Handle browser back button or navigation attempts
   */
  useEffect(() => {
    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      // Show browser confirmation dialog when trying to leave the page
      event.preventDefault();
      event.returnValue = 'Are you sure you want to leave this race? Your progress will be lost.';
      return event.returnValue;
    };

    // Add event listener for page unload
    window.addEventListener('beforeunload', handleBeforeUnload);

    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);

  /**
   * Handle manual navigation to lobby (from overlay button)
   */
  const handleReturnToLobby = () => {
    setShowLeaveDialog(true);
  };

  /**
   * Handle return to lobby from race completion screen
   * Requirements: 7.5
   */
  const handleReturnToLobbyFromCompletion = () => {
    navigate('/game');
  };

  /**
   * Confirm leaving the race
   */
  const confirmLeaveRace = () => {
    setShowLeaveDialog(false);
    navigate('/game');
  };

  /**
   * Cancel leaving the race
   */
  const cancelLeaveRace = () => {
    setShowLeaveDialog(false);
  };

  return (
    <>
      {/* Leave race confirmation dialog */}
      <LeaveRaceDialog
        isOpen={showLeaveDialog}
        onConfirm={confirmLeaveRace}
        onCancel={cancelLeaveRace}
      />

      {/* Return to Lobby button - positioned as overlay */}
      <div className="fixed top-4 left-4 z-40">
        <button
          onClick={handleReturnToLobby}
          className="bg-gray-800 hover:bg-gray-700 text-white px-4 py-2 rounded-lg border border-gray-600 transition-colors shadow-lg"
        >
          ← Return to Lobby
        </button>
      </div>

      {/* Race Container */}
      <RaceContainer
        raceUuid={raceUuid}
        playerUuid={user.uuid}
        onRaceComplete={handleRaceComplete}
        onError={handleRaceError}
        onReturnToLobby={handleReturnToLobbyFromCompletion}
      />
    </>
  );
}
