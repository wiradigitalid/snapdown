export type AnnotationType = 'marker' | 'shape' | 'callout' | 'blur' | 'arrow' | 'text';

export interface VisualShapeAnnotation {
  id: string;
  kind: 'shape';
  x: number; // 0.0 .. 1.0
  y: number; // 0.0 .. 1.0
  width: number; // 0.0 .. 1.0
  height: number; // 0.0 .. 1.0
  strokeColor?: string;
  strokeWidth?: number;
}

export interface VisualArrowAnnotation {
  id: string;
  kind: 'arrow';
  startX: number; // 0.0 .. 1.0
  startY: number; // 0.0 .. 1.0
  endX: number; // 0.0 .. 1.0
  endY: number; // 0.0 .. 1.0
  color?: string;
  strokeWidth?: number;
}

export interface VisualCalloutAnnotation {
  id: string;
  kind: 'callout';
  x: number; // 0.0 .. 1.0
  y: number; // 0.0 .. 1.0
  width: number; // 0.0 .. 1.0
  height: number; // 0.0 .. 1.0
  tailX: number; // 0.0 .. 1.0
  tailY: number; // 0.0 .. 1.0
  text: string;
  fontSize?: number;
  fontFamily?: string;
  fontWeight?: string;
  fontStyle?: string;
  bgColor?: string;
  textColor?: string;
}

export interface VisualBlurAnnotation {
  id: string;
  kind: 'blur';
  x: number; // 0.0 .. 1.0
  y: number; // 0.0 .. 1.0
  width: number; // 0.0 .. 1.0
  height: number; // 0.0 .. 1.0
  blurRadius?: number;
}

export interface VisualTextAnnotation {
  id: string;
  kind: 'text';
  x: number; // 0.0 .. 1.0
  y: number; // 0.0 .. 1.0
  width: number; // 0.0 .. 1.0
  height: number; // 0.0 .. 1.0
  text: string;
  fontSize?: number;
  fontFamily?: string;
  fontWeight?: string;
  fontStyle?: string;
  textColor?: string;
}

export type VisualAnnotationItem =
  | VisualShapeAnnotation
  | VisualArrowAnnotation
  | VisualCalloutAnnotation
  | VisualBlurAnnotation
  | VisualTextAnnotation;
