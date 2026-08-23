import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { QualityBudgetSection } from '../components/QualityBudgetSection';
import { QualityBudget, LatestFindingAttributionDto } from '../types/settings';

describe('Quality Budget Intent and Derivation UI (W6-S4 / DEC-004 / SCN-03 / FR-5)', () => {
  const defaultQualityBudget: QualityBudget = {
    named: 'auto',
    prose: 'Sizes each capture to what it is. Most captures land near 120 KB.',
    max_long_edge: 1600,
    encoder_quality: 82,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('a_reviewer_who_never_opens_advanced_never_sees_a_raw_number', () => {
    const handleSave = vi.fn().mockResolvedValue(undefined);

    render(
      <QualityBudgetSection
        qualityBudget={defaultQualityBudget}
        latestFindingSize={null}
        latestFinding={null}
        onSaveQualityBudget={handleSave}
      />
    );

    const section = screen.getByTestId('quality-budget-section');

    // 1. Four default preset options are rendered in SegmentedControl
    expect(within(section).getByRole('radio', { name: 'Auto' })).toBeInTheDocument();
    expect(within(section).getByRole('radio', { name: 'Sharp' })).toBeInTheDocument();
    expect(within(section).getByRole('radio', { name: 'Balanced' })).toBeInTheDocument();
    expect(within(section).getByRole('radio', { name: 'Small' })).toBeInTheDocument();

    // 2. Auto is active and its prose is shown
    expect(within(section).getByRole('radio', { name: 'Auto' })).toHaveAttribute('aria-checked', 'true');
    expect(
      screen.getByText('Sizes each capture to what it is. Most captures land near 120 KB.')
    ).toBeInTheDocument();

    // 3. DEC-004 core requirement: Raw number inputs and labels MUST NOT exist in DOM when Advanced is closed
    expect(screen.queryByLabelText(/Max Long Edge/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/Encoder Quality/i)).not.toBeInTheDocument();
    expect(screen.queryByTestId('advanced-fields')).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue('1600')).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue('75')).not.toBeInTheDocument();
    expect(screen.queryByDisplayValue('82')).not.toBeInTheDocument();

    // 4. Advanced disclosure button is present
    expect(screen.getByTestId('advanced-disclosure-button')).toBeInTheDocument();
  });

  it('editing_an_advanced_value_moves_the_control_to_custom_visibly', async () => {
    const handleSave = vi.fn().mockResolvedValue(undefined);

    render(
      <QualityBudgetSection
        qualityBudget={defaultQualityBudget}
        latestFindingSize={null}
        latestFinding={null}
        onSaveQualityBudget={handleSave}
      />
    );

    // Open Advanced disclosure
    const advancedBtn = screen.getByTestId('advanced-disclosure-button');
    fireEvent.click(advancedBtn);

    // Verify fields are now visible
    expect(screen.getByTestId('advanced-fields')).toBeInTheDocument();
    const edgeInput = screen.getByLabelText('Max Long Edge (px)');
    const qualityInput = screen.getByLabelText('Encoder Quality (10-100)');
    expect(edgeInput).toBeInTheDocument();
    expect(qualityInput).toBeInTheDocument();

    // Edit Max Long Edge to 2048
    fireEvent.change(edgeInput, { target: { value: '2048' } });

    await waitFor(() => {
      // SegmentedControl now has segment "Custom" and it is checked (BR-117)
      const customRadio = screen.getByRole('radio', { name: 'Custom' });
      expect(customRadio).toBeInTheDocument();
      expect(customRadio).toHaveAttribute('aria-checked', 'true');

      // Prose updates to Custom prose
      expect(screen.getByText('Custom limits set in Advanced.')).toBeInTheDocument();

      // Atomic save invoked with 'custom' and validated pair (BR-116)
      expect(handleSave).toHaveBeenCalledWith('custom', {
        max_long_edge: 2048,
        encoder_quality: 82,
      });
    });
  });

  it('selecting_a_preset_updates_prose_and_saves_atomically', async () => {
    const handleSave = vi.fn().mockResolvedValue(undefined);

    render(
      <QualityBudgetSection
        qualityBudget={defaultQualityBudget}
        latestFindingSize={null}
        latestFinding={null}
        onSaveQualityBudget={handleSave}
      />
    );

    // Select Sharp preset
    const sharpBtn = screen.getByRole('radio', { name: 'Sharp' });
    fireEvent.click(sharpBtn);

    await waitFor(() => {
      expect(handleSave).toHaveBeenCalledWith('sharp', null);
      expect(screen.getByText('Keeps small text crisp. Files are larger.')).toBeInTheDocument();
    });

    // Select Small preset
    const smallBtn = screen.getByRole('radio', { name: 'Small' });
    fireEvent.click(smallBtn);

    await waitFor(() => {
      expect(handleSave).toHaveBeenCalledWith('small', null);
      expect(screen.getByText('The smallest file that is still readable.')).toBeInTheDocument();
    });

    // Select Balanced preset
    const balancedBtn = screen.getByRole('radio', { name: 'Balanced' });
    fireEvent.click(balancedBtn);

    await waitFor(() => {
      expect(handleSave).toHaveBeenCalledWith('balanced', null);
      expect(
        screen.getByText('A middle setting that does not change with the capture.')
      ).toBeInTheDocument();
    });
  });

  it('the_readout_names_the_budget_that_produced_the_latest_finding', () => {
    const handleSave = vi.fn().mockResolvedValue(undefined);

    const latestFinding: LatestFindingAttributionDto = {
      size_bytes: 188416, // ~184 KB
      width: 1408,
      height: 792,
      budget_name: 'Auto',
    };

    const { rerender } = render(
      <QualityBudgetSection
        qualityBudget={defaultQualityBudget}
        latestFindingSize={188416}
        latestFinding={latestFinding}
        onSaveQualityBudget={handleSave}
      />
    );

    const readout = screen.getByTestId('latest-finding-size');
    expect(readout).toHaveTextContent('Latest: 184.0 KB · 1408 px · Auto');

    // Rerender with no captures
    rerender(
      <QualityBudgetSection
        qualityBudget={defaultQualityBudget}
        latestFindingSize={null}
        latestFinding={null}
        onSaveQualityBudget={handleSave}
      />
    );

    expect(screen.getByTestId('latest-finding-size')).toHaveTextContent('No captures yet');
  });

  it('invalid_advanced_values_are_refused_and_show_error', async () => {
    const handleSave = vi.fn().mockResolvedValue(undefined);

    render(
      <QualityBudgetSection
        qualityBudget={defaultQualityBudget}
        latestFindingSize={null}
        latestFinding={null}
        onSaveQualityBudget={handleSave}
      />
    );

    // Open Advanced
    fireEvent.click(screen.getByTestId('advanced-disclosure-button'));

    const edgeInput = screen.getByLabelText('Max Long Edge (px)');
    const qualityInput = screen.getByLabelText('Encoder Quality (10-100)');

    // Enter out-of-range edge (< 320)
    fireEvent.change(edgeInput, { target: { value: '100' } });

    await waitFor(() => {
      expect(screen.getByTestId('quality-budget-error')).toHaveTextContent(
        'Max long edge must be between 320 and 7680 px'
      );
      expect(handleSave).not.toHaveBeenCalled();
    });

    handleSave.mockClear();

    // Enter out-of-range quality (> 100)
    fireEvent.change(qualityInput, { target: { value: '105' } });

    await waitFor(() => {
      expect(screen.getByTestId('quality-budget-error')).toHaveTextContent(
        'Quality must be between 10% and 100%'
      );
      expect(handleSave).not.toHaveBeenCalled();
    });
  });
});