import React from 'react';

interface SkeletonLoaderProps {
  width?: string | number;
  height?: string | number;
  className?: string;
  variant?: 'text' | 'rectangular' | 'circular';
  lines?: number; // For text variant
  animate?: boolean;
}

/**
 * Skeleton loader component for better loading UX
 * Requirements: 3.4 - Add skeleton loaders for data fetching
 */
export const SkeletonLoader: React.FC<SkeletonLoaderProps> = ({
  width = '100%',
  height = '1rem',
  className = '',
  variant = 'rectangular',
  lines = 1,
  animate = true,
}) => {
  const baseClasses = `bg-gray-700 ${animate ? 'animate-pulse' : ''}`;

  const variantClasses = {
    text: 'rounded',
    rectangular: 'rounded',
    circular: 'rounded-full',
  };

  const style = {
    width: typeof width === 'number' ? `${width}px` : width,
    height: typeof height === 'number' ? `${height}px` : height,
  };

  if (variant === 'text' && lines > 1) {
    return (
      <div className={`space-y-2 ${className}`}>
        {Array.from({ length: lines }).map((_, index) => (
          <div
            key={index}
            className={`${baseClasses} ${variantClasses[variant]}`}
            style={{
              ...style,
              width: index === lines - 1 ? '75%' : style.width, // Last line is shorter
            }}
          />
        ))}
      </div>
    );
  }

  return <div className={`${baseClasses} ${variantClasses[variant]} ${className}`} style={style} />;
};

/**
 * Pre-built skeleton components for common use cases
 */
export const SkeletonCard: React.FC<{ className?: string }> = ({ className = '' }) => (
  <div className={`bg-gray-800 rounded-lg p-4 ${className}`}>
    <SkeletonLoader variant="text" height="1.5rem" width="60%" className="mb-3" />
    <SkeletonLoader variant="text" lines={3} className="mb-4" />
    <div className="flex gap-2">
      <SkeletonLoader width="80px" height="32px" />
      <SkeletonLoader width="80px" height="32px" />
    </div>
  </div>
);

export const SkeletonButton: React.FC<{ className?: string }> = ({ className = '' }) => (
  <SkeletonLoader width="120px" height="40px" className={`rounded-lg ${className}`} />
);

export const SkeletonAvatar: React.FC<{ size?: number; className?: string }> = ({
  size = 40,
  className = '',
}) => <SkeletonLoader variant="circular" width={size} height={size} className={className} />;

export const SkeletonTable: React.FC<{
  rows?: number;
  columns?: number;
  className?: string;
}> = ({ rows = 3, columns = 4, className = '' }) => (
  <div className={`space-y-3 ${className}`}>
    {Array.from({ length: rows }).map((_, rowIndex) => (
      <div key={rowIndex} className="flex gap-4">
        {Array.from({ length: columns }).map((_, colIndex) => (
          <SkeletonLoader key={colIndex} height="2rem" width={colIndex === 0 ? '25%' : '20%'} />
        ))}
      </div>
    ))}
  </div>
);

export default SkeletonLoader;
