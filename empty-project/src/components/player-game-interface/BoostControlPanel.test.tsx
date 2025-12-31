import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { BoostControlPanel } from './BoostControlPanel';
import type { TurnPhase } from '../../types/race';

describe('BoostControlPanel', () => {
  const defaultProps = {
    selectedBoost: null,
    availableBoosts: [0, 1, 2, 3, 4],
    onBoostSelect: vi.fn(),
    onValidateTurn: vi.fn(),
    isSubmitting: false,
    hasSubmitted: false,
    turnPhase: 'WaitingForPlayers' as TurnPhase,
  };

  it('renders boost control panel with header', () => {
    render(<BoostControlPanel {...defaultProps} />);

    expect(screen.getByText('Boost Control')).toBeInTheDocument();
    expect(screen.getByText('Select your boost value for this turn')).toBeInTheDocument();
  });

  it('renders all boost buttons (0-4)', () => {
    render(<BoostControlPanel {...defaultProps} />);

    for (let i = 0; i <= 4; i++) {
      expect(screen.getByRole('button', { name: `Select boost value ${i}` })).toBeInTheDocument();
    }
  });

  it('calls onBoostSelect when available boost button is clicked', () => {
    const onBoostSelect = vi.fn();
    render(<BoostControlPanel {...defaultProps} onBoostSelect={onBoostSelect} />);

    const boostButton = screen.getByRole('button', { name: 'Select boost value 2' });
    fireEvent.click(boostButton);

    expect(onBoostSelect).toHaveBeenCalledWith(2);
  });

  it('does not call onBoostSelect for unavailable boost', () => {
    const onBoostSelect = vi.fn();
    render(
      <BoostControlPanel
        {...defaultProps}
        availableBoosts={[0, 1, 3, 4]} // 2 is not available
        onBoostSelect={onBoostSelect}
      />,
    );

    const boostButton = screen.getByRole('button', { name: 'Select boost value 2' });
    fireEvent.click(boostButton);

    expect(onBoostSelect).not.toHaveBeenCalled();
  });

  it('shows selected boost information', () => {
    render(<BoostControlPanel {...defaultProps} selectedBoost={3} />);

    expect(screen.getByText('Selected Boost:')).toBeInTheDocument();
    // Use more specific selector for the selected boost display
    expect(screen.getByText('✓ Available')).toBeInTheDocument();

    // Check that the button is marked as selected
    const selectedButton = screen.getByRole('button', { name: 'Select boost value 3' });
    expect(selectedButton).toHaveAttribute('aria-pressed', 'true');
  });

  it('shows validate turn button when boost is selected', () => {
    render(<BoostControlPanel {...defaultProps} selectedBoost={2} />);

    expect(
      screen.getByRole('button', { name: 'Validate turn with selected boost' }),
    ).toBeInTheDocument();
  });

  it('disables validate button when no boost is selected', () => {
    render(<BoostControlPanel {...defaultProps} selectedBoost={null} />);

    const validateButton = screen.getByRole('button', {
      name: 'Validate turn with selected boost',
    });
    expect(validateButton).toBeDisabled();
  });

  it('shows confirmation dialog when validate button is clicked', () => {
    render(<BoostControlPanel {...defaultProps} selectedBoost={2} />);

    const validateButton = screen.getByRole('button', {
      name: 'Validate turn with selected boost',
    });
    fireEvent.click(validateButton);

    expect(screen.getByText('Confirm Turn Validation')).toBeInTheDocument();
    expect(
      screen.getByText('You are about to validate your turn with boost value:'),
    ).toBeInTheDocument();
  });

  it('calls onValidateTurn when confirmation is clicked', () => {
    const onValidateTurn = vi.fn();
    render(
      <BoostControlPanel {...defaultProps} selectedBoost={2} onValidateTurn={onValidateTurn} />,
    );

    // Click validate button to show confirmation
    const validateButton = screen.getByRole('button', {
      name: 'Validate turn with selected boost',
    });
    fireEvent.click(validateButton);

    // Click confirm button
    const confirmButton = screen.getByRole('button', { name: 'Confirm' });
    fireEvent.click(confirmButton);

    expect(onValidateTurn).toHaveBeenCalled();
  });

  it('shows submitted state when hasSubmitted is true', () => {
    render(<BoostControlPanel {...defaultProps} selectedBoost={3} hasSubmitted={true} />);

    expect(screen.getByText('Turn Validated')).toBeInTheDocument();
    expect(screen.getByText('Waiting for other players...')).toBeInTheDocument();
  });

  it('disables interactions when turn phase is not WaitingForPlayers', () => {
    render(<BoostControlPanel {...defaultProps} turnPhase="Processing" />);

    expect(screen.getByText('Turn actions not available')).toBeInTheDocument();
    expect(screen.getByText('Current phase: Processing')).toBeInTheDocument();
  });

  it('shows used indicator for unavailable boosts', () => {
    render(
      <BoostControlPanel
        {...defaultProps}
        availableBoosts={[0, 1, 3, 4]} // 2 is not available
      />,
    );

    // The "Used" badge should be visible for boost 2
    expect(screen.getByText('Used')).toBeInTheDocument();
  });
});
