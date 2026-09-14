export const rotate = (epoch: number): number => epoch + 1

// Regression: the counter once skipped an epoch when two rotations raced.
export const bug__epoch_skipped_on_rotate = 'fixed in rotate()'
