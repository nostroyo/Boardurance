import React, { useEffect, useState, useCallback } from 'react';

export interface CarMovement {
  participantUuid: string;
  fromSector: number;
  toSector: number;
  fromPosition: number;
  toPosition: number;
  movementType: 'Forward' | 'Backward' | 'Stay';
}

export interface CarMovementAnimationProps {
  movements: CarMovement[];
  duration?: number;
  onAnimationComplete?: () => void;
  children: React.ReactNode;
}

export interface AnimationState {
  isAnimating: boolean;
  currentMovements: CarMovement[];
  progress: number;
}

const CarMovementAnimationComponent: React.FC<CarMovementAnimationProps> = ({
  movements,
  duration = 1000,
  onAnimationComplete,
  children,
}) => {
  const [animationState, setAnimationState] = useState<AnimationState>({
    isAnimating: false,
    currentMovements: [],
    progress: 0,
  });

  // Start animation when movements change
  useEffect(() => {
    if (movements.length > 0) {
      startAnimation(movements);
    }
  }, [movements]);

  const startAnimation = useCallback((newMovements: CarMovement[]) => {
    setAnimationState({
      isAnimating: true,
      currentMovements: newMovements,
      progress: 0,
    });

    // Animate progress from 0 to 1
    const startTime = Date.now();
    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);

      setAnimationState((prev) => ({
        ...prev,
        progress,
      }));

      if (progress < 1) {
        requestAnimationFrame(animate);
      } else {
        // Animation complete
        setAnimationState({
          isAnimating: false,
          currentMovements: [],
          progress: 1,
        });
        onAnimationComplete?.();
      }
    };

    requestAnimationFrame(animate);
  }, [duration, onAnimationComplete]);

  // Calculate easing function for smooth animation
  const easeInOutCubic = (t: number): number => {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  };

  // Get animation transform for a specific participant
  const getAnimationTransform = (participantUuid: string): string => {
    if (!animationState.isAnimating) return '';

    const movement = animationState.currentMovements.find(
      (m) => m.participantUuid === participantUuid
    );

    if (!movement || movement.movementType === 'Stay') return '';

    const easedProgress = easeInOutCubic(animationState.progress);
    
    // Calculate movement direction and distance
    const sectorDifference = movement.toSector - movement.fromSector;
    const positionDifference = movement.toPosition - movement.fromPosition;
    
    // Translate based on sector and position changes
    const sectorOffset = sectorDifference * 100; // 100px per sector (adjust as needed)
    const positionOffset = positionDifference * 60; // 60px per position slot
    
    const totalOffsetX = sectorOffset + positionOffset;
    const currentOffsetX = totalOffsetX * easedProgress;
    
    // Add slight vertical bounce for visual appeal
    const bounceHeight = Math.sin(easedProgress * Math.PI) * 10;
    
    return `translateX(${currentOffsetX}px) translateY(${-bounceHeight}px)`;
  };

  // Provide animation context to children
  const animationContext = {
    isAnimating: animationState.isAnimating,
    progress: animationState.progress,
    getTransform: getAnimationTransform,
    movements: animationState.currentMovements,
  };

  return (
    <div className="relative">
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child)) {
          return React.cloneElement(child, {
            ...(child.props as any),
            animationContext,
          });
        }
        return child;
      })}
    </div>
  );
};

export const CarMovementAnimation = React.memo(CarMovementAnimationComponent);

// Hook for using car movement animations
export const useCarMovementAnimation = () => {
  const [movements, setMovements] = useState<CarMovement[]>([]);
  const [isAnimating, setIsAnimating] = useState(false);

  const startMovement = useCallback((newMovements: CarMovement[]) => {
    setMovements(newMovements);
    setIsAnimating(true);
  }, []);

  const onAnimationComplete = useCallback(() => {
    setIsAnimating(false);
    setMovements([]);
  }, []);

  return {
    movements,
    isAnimating,
    startMovement,
    onAnimationComplete,
  };
};