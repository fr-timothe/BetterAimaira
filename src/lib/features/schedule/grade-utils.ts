import type { GradeBlock, GradeCourse, GradeEvaluation, GradePeriod } from './types';

/** Normalisation target: the portal mixes /20 scales with bare marks. */
const REFERENCE_SCALE = 20;

export type ScoredEvaluation = {
  value: number | null;
  max: number;
  /** Percentage of the course mark, when the portal states one. */
  weight: number | null;
};

export function evaluationScore(evaluation: GradeEvaluation): ScoredEvaluation {
  const value = parseDecimal(evaluation.score);
  const max = parseDecimal(evaluation.scale) ?? REFERENCE_SCALE;
  return {
    value,
    max: max > 0 ? max : REFERENCE_SCALE,
    weight: parseWeightPercent(evaluation.weight),
  };
}

/**
 * `50,00%` is a share of the course mark; `Pondération : 20,00` is the raw
 * weight of a sub-evaluation inside its parent. Only the former is a share, so
 * only the former is returned.
 */
export function parseWeightPercent(weight: string | null): number | null {
  if (!weight) return null;
  const match = weight.match(/(\d+(?:[.,]\d+)?)\s*%/);
  if (!match) return null;
  return parseDecimal(match[1]);
}

export function scaledValue(evaluation: GradeEvaluation): number | null {
  const { value, max } = evaluationScore(evaluation);
  if (value === null) return null;
  return (value / max) * REFERENCE_SCALE;
}

/**
 * Sub-evaluations are already rolled up into their parent by the portal, so
 * only top-level entries take part in the average.
 */
export function courseAverage(course: GradeCourse): number | null {
  const scored = course.evaluations
    .map((evaluation) => ({ value: scaledValue(evaluation), weight: parseWeightPercent(evaluation.weight) }))
    .filter((entry): entry is { value: number; weight: number | null } => entry.value !== null);
  if (scored.length === 0) return null;

  const totalWeight = scored.reduce((total, entry) => total + (entry.weight ?? 0), 0);
  if (totalWeight > 0) {
    const weighted = scored.reduce((total, entry) => total + entry.value * (entry.weight ?? 0), 0);
    return weighted / totalWeight;
  }

  return scored.reduce((total, entry) => total + entry.value, 0) / scored.length;
}

export function blockCourses(block: GradeBlock): GradeCourse[] {
  return block.sections.flatMap((section) => section.courses);
}

export function periodCourses(period: GradePeriod): GradeCourse[] {
  return period.blocks.flatMap(blockCourses);
}

/** Unweighted: the portal exposes no credit count to weight courses with. */
export function averageOfCourses(courses: GradeCourse[]): number | null {
  const averages = courses
    .map(courseAverage)
    .filter((average): average is number => average !== null);
  if (averages.length === 0) return null;
  return averages.reduce((total, average) => total + average, 0) / averages.length;
}

export function gradedEvaluations(courses: GradeCourse[]): GradeEvaluation[] {
  return courses.flatMap((course) =>
    course.evaluations.filter((evaluation) => scaledValue(evaluation) !== null)
  );
}

export function extremeScaledValue(
  courses: GradeCourse[],
  pick: 'highest' | 'lowest'
): number | null {
  const values = gradedEvaluations(courses)
    .map(scaledValue)
    .filter((value): value is number => value !== null);
  if (values.length === 0) return null;
  return pick === 'highest' ? Math.max(...values) : Math.min(...values);
}

/**
 * Running mean over the evaluations in portal order — the only progression the
 * grades page supports, since it publishes no date next to a mark.
 */
export function cumulativeAverages(courses: GradeCourse[]): number[] {
  const series: number[] = [];
  let total = 0;

  for (const evaluation of gradedEvaluations(courses)) {
    const value = scaledValue(evaluation);
    if (value === null) continue;
    total += value;
    series.push(total / (series.length + 1));
  }

  return series;
}

function parseDecimal(value: string | null): number | null {
  if (!value) return null;
  const parsed = Number.parseFloat(value.replace(',', '.').replace(/[^0-9.-]/g, ''));
  return Number.isNaN(parsed) ? null : parsed;
}
