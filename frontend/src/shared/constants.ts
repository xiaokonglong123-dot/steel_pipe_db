/** Shared domain constants — single source of truth */

/** API 5CT steel pipe grades */
export const API_5CT_GRADES = [
  'H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125',
] as const;

/** Pipe classification types */
export const PIPE_TYPES = ['seamless', 'screen'] as const;

/** Pipe category types (includes sub-categories) */
export const PIPE_CATEGORIES = ['seamless', 'casing', 'tubing', 'screen'] as const;

/** Inbound order types */
export const INBOUND_TYPES = ['purchase', 'production', 'return'] as const;

/** Outbound order types */
export const OUTBOUND_TYPES = ['sales', 'sample', 'scrap'] as const;

/** Contract types */
export const CONTRACT_TYPES = ['purchase', 'sales'] as const;

/** Quality certificate statuses */
export const CERT_STATUSES = ['draft', 'active', 'void'] as const;

/** Quality check results */
export const CHECK_RESULTS = ['pass', 'fail', 'pending'] as const;

/** Order statuses */
export const ORDER_STATUSES = ['draft', 'confirmed', 'completed', 'cancelled'] as const;

/** User roles */
export const USER_ROLES = ['admin', 'warehouse', 'quality', 'sales'] as const;

/** Location types */
export const LOCATION_TYPES = ['warehouse', 'yard', 'dock'] as const;

/** Inventory check statuses */
export const CHECK_LIST_STATUSES = ['draft', 'in_progress', 'completed'] as const;
