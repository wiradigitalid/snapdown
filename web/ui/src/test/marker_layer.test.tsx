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

describe('MarkerLayer Component', () => {
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
});
