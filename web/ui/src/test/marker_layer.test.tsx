import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { MarkerLayer } from '../components/MarkerLayer';

const mockMarkers = [
  {
    id: 'm1',
    finding_id: 'fid-1',
    ordinal: 1,
    x: 0.25,
    y: 0.5,
    comment: 'First issue',
  },
  {
    id: 'm2',
    finding_id: 'fid-1',
    ordinal: 2,
    x: 0.75,
    y: 0.8,
    comment: 'Second issue',
  },
];

describe('MarkerLayer Component (LC-007)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('marker_placement_normalizes_coordinates', () => {
    const onAddMarker = vi.fn();

    render(
      <MarkerLayer
        markers={mockMarkers}
        onAddMarker={onAddMarker}
      />
    );

    const layer = screen.getByTestId('marker-layer');
    expect(layer).toBeInTheDocument();

    // Verify existing badges rendered
    expect(screen.getByTestId('marker-badge-1')).toBeInTheDocument();
    expect(screen.getByTestId('marker-badge-2')).toBeInTheDocument();

    // Mock getBoundingClientRect
    vi.spyOn(layer, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      top: 0,
      width: 400,
      height: 300,
      right: 400,
      bottom: 300,
      x: 0,
      y: 0,
      toJSON: () => {},
    });

    // Click at clientX: 200 (50%), clientY: 150 (50%)
    fireEvent.click(layer, { clientX: 200, clientY: 150 });

    expect(onAddMarker).toHaveBeenCalledWith(0.5, 0.5);
  });

  it('dragging_marker_updates_position', () => {
    const onUpdateMarkerPosition = vi.fn();
    const onSelectMarker = vi.fn();

    render(
      <MarkerLayer
        markers={mockMarkers}
        onUpdateMarkerPosition={onUpdateMarkerPosition}
        onSelectMarker={onSelectMarker}
      />
    );

    const layer = screen.getByTestId('marker-layer');
    vi.spyOn(layer, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      top: 0,
      width: 500,
      height: 400,
      right: 500,
      bottom: 400,
      x: 0,
      y: 0,
      toJSON: () => {},
    });

    const badge1 = screen.getByTestId('marker-badge-1');
    fireEvent.mouseDown(badge1);

    expect(onSelectMarker).toHaveBeenCalledWith('m1');

    // Move mouse on window
    fireEvent.mouseMove(window, { clientX: 250, clientY: 200 });
    expect(onUpdateMarkerPosition).toHaveBeenCalledWith('m1', 0.5, 0.5);

    fireEvent.mouseUp(window);
  });

  it('keyboard_navigation_on_markers', () => {
    const onSelectMarker = vi.fn();
    const onDeleteMarker = vi.fn();

    render(
      <MarkerLayer
        markers={mockMarkers}
        onSelectMarker={onSelectMarker}
        onDeleteMarker={onDeleteMarker}
      />
    );

    const badge1 = screen.getByRole('button', { name: 'Marker 1' });
    fireEvent.keyDown(badge1, { key: 'Enter' });
    expect(onSelectMarker).toHaveBeenCalledWith('m1');

    fireEvent.keyDown(badge1, { key: 'Delete' });
    expect(onDeleteMarker).toHaveBeenCalledWith('m1');
  });
});
