import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { BoostSelector } from './BoostSelector';

describe('BoostSelector', () => {
  const defaultProps = {
    selectedBoost: null,
    availableBoosts: [0, 1, 2, 3, 4],
    onBoostSelect: vi.fn(),
    onSubmit: vi.fn(),
    isSubmitting: false,
    hasSubmitted: false,
  };

  it('renders all boost value buttons', () => {
    render(<BoostSelector {...defaultProps} />);

    for (let i = 0; i <= 4; i++) {
      expect(screen.getByLabelText(`Select boost value ${i}`)).toBeInTheDocument();
    }
  });

  it('shows "Used" badge on unavailable boost cards', () => {
    render(<BoostSelector {...defaultProps} availableBoosts={[1, 2, 3, 4]} />);

    expect(screen.getByText('Used')).toBeInTheDocument();
  });

  it('allows selection of available boost cards', () => {
    const onBoostSelect = vi.fn();
    render(<BoostSelector {...defaultProps} onBoostSelect={onBoostSelect} />);

    const boost2Button = screen.getByLabelText('Select boost value 2');
    fireEvent.click(boost2Button);

    expect(onBoostSelect).toHaveBeenCalledWith(2);
  });

  it('prevents selection of unavailable boost cards', () => {
    const onBoostSelect = vi.fn();
    render(
      <BoostSelector
        {...defaultProps}
        availableBoosts={[1, 2, 3, 4]}
        onBoostSelect={onBoostSelect}
      />,
    );

    const boost0Button = screen.getByLabelText('Select boost value 0');
    fireEvent.click(boost0Button);

    expect(onBoostSelect).not.toHaveBeenCalled();
  });

  it('disables submit button when no boost is selected', () => {
    render(<BoostSelector {...defaultProps} />);

    const submitButton = screen.getByLabelText('Submit boost selection');
    expect(submitButton).toBeDisabled();
  });

  it('enables submit button when available boost is selected', () => {
    render(<BoostSelector {...defaultProps} selectedBoost={2} />);

    const submitButton = screen.getByLabelText('Submit boost selection');
    expect(submitButton).not.toBeDisabled();
  });

  it('shows confirmation dialog when submit is clicked', () => {
    render(<BoostSelector {...defaultProps} selectedBoost={2} />);

    const submitButton = screen.getByLabelText('Submit boost selection');
    fireEvent.click(submitButton);

    expect(screen.getByText('Confirm Action')).toBeInTheDocument();
    expect(screen.getByText(/You are about to submit boost value:/)).toBeInTheDocument();
  });

  it('calls onSubmit when confirmation is confirmed', () => {
    const onSubmit = vi.fn();
    render(<BoostSelector {...defaultProps} selectedBoost={2} onSubmit={onSubmit} />);

    const submitButton = screen.getByLabelText('Submit boost selection');
    fireEvent.click(submitButton);

    const confirmButton = screen.getByText('Confirm');
    fireEvent.click(confirmButton);

    expect(onSubmit).toHaveBeenCalled();
  });

  it('shows loading state during submission', () => {
    render(<BoostSelector {...defaultProps} selectedBoost={2} isSubmitting={true} />);

    // Click submit to show confirmation
    const submitButton = screen.getByLabelText('Submit boost selection');
    fireEvent.click(submitButton);

    expect(screen.getByText('Submitting...')).toBeInTheDocument();
  });

  it('shows "Action Submitted" state after submission', () => {
    render(<BoostSelector {...defaultProps} selectedBoost={2} hasSubmitted={true} />);

    expect(screen.getByText('Action Submitted')).toBeInTheDocument();
    expect(screen.getByText('Waiting for other players...')).toBeInTheDocument();
  });

  it('prevents boost selection after submission', () => {
    const onBoostSelect = vi.fn();
    render(
      <BoostSelector
        {...defaultProps}
        selectedBoost={2}
        hasSubmitted={true}
        onBoostSelect={onBoostSelect}
      />,
    );

    const boost3Button = screen.getByLabelText('Select boost value 3');
    fireEvent.click(boost3Button);

    expect(onBoostSelect).not.toHaveBeenCalled();
  });

  it('shows validation message for unavailable selected boost', () => {
    render(<BoostSelector {...defaultProps} selectedBoost={0} availableBoosts={[1, 2, 3, 4]} />);

    expect(screen.getByText(/This boost card has already been used/)).toBeInTheDocument();
  });
});
