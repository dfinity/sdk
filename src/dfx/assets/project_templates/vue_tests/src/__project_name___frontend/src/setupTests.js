import matchers from '@testing-library/jest-dom/matchers';
import 'cross-fetch/polyfill';
import { expect } from 'vitest';

expect.extend(matchers);
