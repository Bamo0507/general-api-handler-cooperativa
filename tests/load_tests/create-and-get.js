import http from 'k6/http';
import { check, sleep } from 'k6';

// Configuración fija para ejecución local (no placeholders)
const BASE_HTTP = 'http://localhost:8080';
const GRAPHQL_ENDPOINT = `${BASE_HTTP}/graphql/payment`;
const GRAPHQL_LOAN_ENDPOINT = `${BASE_HTTP}/graphql/loan`;
const GRAPHQL_FINE_ENDPOINT = `${BASE_HTTP}/graphql/fine`;
const GRAPHQL_QUOTA_ENDPOINT = `${BASE_HTTP}/graphql/quota`;
const SIGNUP_ENDPOINT = `${BASE_HTTP}/general/signup`;

export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 5,
      duration: '30s',
      exec: 'smokeScenario',
    },
    full: {
      executor: 'ramping-vus',
      startTime: '35s', // start after smoke
      stages: [
        { duration: '1m', target: 10 },
        { duration: '2m', target: 50 },
        { duration: '1m', target: 100 },
        { duration: '1m', target: 0 },
      ],
      exec: 'fullScenario',
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<1000'],
    checks: ['rate>0.99'],
  },
};


// Helper: create a unique username
function uniqueUsername() {
  return `k6_user_${__VU}_${Date.now()}`;
}

// Signup via REST to obtain access_token (TokenInfo.access_token)
function signupAndGetToken() {
  const user = uniqueUsername();
  const payload = JSON.stringify({
    user_name: user,
    pass_code: 'testpass123',
    real_name: 'k6 test user'
  });

  const res = http.post(SIGNUP_ENDPOINT, payload, { headers: { 'Content-Type': 'application/json' } });
  check(res, { 'signup status 200': (r) => r.status === 200 });

  let token = null;
  try {
    const body = res.json();
    // The server returns a Result serialized; it can be { Ok: { access_token: '...' } }
    if (body && body.access_token) {
      token = body.access_token;
    } else if (body && body.Ok && body.Ok.access_token) {
      token = body.Ok.access_token;
    } else if (body && body.ok && body.ok.access_token) {
      token = body.ok.access_token;
    } else if (body && body[0] && body[0].access_token) {
      token = body[0].access_token;
    } else {
      token = null;
    }
  } catch (e) {
    token = null;
  }
  return token;
}

// GraphQL helpers
function graphqlRequest(endpoint, query, variables, headers) {
  const payload = JSON.stringify({ query, variables });
  return http.post(endpoint, payload, headers);
}

// Mutation string for createUserPayment (juniper maps snake_case to camelCase)
const CREATE_PAYMENT_MUTATION = `mutation CreatePayment($accessToken: String!, $name: String!, $totalAmount: Float!, $ticketNumber: String!, $accountNumber: String!, $beingPayed: [PayedToInput!]!) {
  createUserPayment(accessToken: $accessToken, name: $name, totalAmount: $totalAmount, ticketNumber: $ticketNumber, accountNumber: $accountNumber, beingPayed: $beingPayed)
}`;

// Mutation to approve or reject a payment
const APPROVE_REJECT_MUTATION = `mutation ApproveOrReject($id: String!, $newState: String!, $commentary: String!) { approveOrRejectPayment(id: $id, newState: $newState, commentary: $commentary) { id name state } }`;

const GET_ALL_PAYMENTS_QUERY = `query GetAllPayments { getAllPayments { id name accountNum } }`;
const GET_USER_PAYMENTS_QUERY = `query GetUserPayments($accessToken: String!) { getUsersPayments(accessToken: $accessToken) { id name totalAmount accountNum } }`;
const GET_HISTORY_QUERY = `query GetHistory($accessToken: String!) { getHistory(accessToken: $accessToken) { payedToCapital owedCapital } }`;

// Loan
const GET_USER_LOANS_QUERY = `query GetUserLoans($accessToken: String!) { getUserLoans(accessToken: $accessToken) { id quotas payed debt total status reason } }`;

// Fine
const GET_FINES_BY_ID_QUERY = `query GetFines($accessToken: String!) { getFinesById(accessToken: $accessToken) { id amount status reason } }`;
const CREATE_FINE_MUTATION = `mutation CreateFine($affiliateKey:String!, $amount:Float!, $motive:String!){ createFine(affiliateKey:$affiliateKey, amount:$amount, motive:$motive) }`;

// Quota
const GET_PENDING_QUOTAS_QUERY = `query GetPendingQuotas($accessToken:String!){ getPendingQuotas(accessToken:$accessToken) { userId amount expDate identifier } }`;
const GET_MONTHLY_AFFILIATE_QUOTA_QUERY = `query GetMonthlyAffiliateQuota($accessToken:String!){ getMonthlyAffiliateQuota(accessToken:$accessToken) { userId amount expDate identifier } }`;
const GET_QUOTAS_PRESTAMO_PENDIENTES_QUERY = `query GetQuotasPrestamoPendientes($accessToken:String!){ getQuotasPrestamoPendientes(accessToken:$accessToken) { userId amount expDate loanId quotaNumber } }`;
const GET_PENDING_LOANS_QUOTAS_QUERY = `query GetPendingLoansQuotas($accessToken:String!){ getPendingLoansQuotas(accessToken:$accessToken) { userId amount expDate loanId quotaNumber nombrePrestamo nombreUsuario } }`;

// Payment additional
const GET_ALL_MEMBERS_QUERY = `query GetAllMembers { getAllMembers { userId name } }`;

// Fine additional
const EDIT_FINE_MUTATION = `mutation EditFine($fineKey:String!, $newAmount:Float, $newMotive:String, $newStatus:String){ editFine(fineKey:$fineKey, newAmount:$newAmount, newMotive:$newMotive, newStatus:$newStatus) }`;

function performCreatePayment(accessToken) {
  const uniqueId = Date.now();
  const variables = {
    accessToken: accessToken,
    name: `k6_payment_${uniqueId}`,
    totalAmount: 42.5,
    ticketNumber: `T_${uniqueId}`,
    accountNumber: `A_${uniqueId}`,
    beingPayed: [ { modelType: 'LOAN', amount: 42.5, modelKey: `LOAN_${uniqueId}` } ]
  };

  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_ENDPOINT, CREATE_PAYMENT_MUTATION, variables, headers);
  check(res, { 'create payment 200': (r) => r.status === 200 });
  // Try to extract payment id from GraphQL response (may be a string id or wrapped)
  try {
    const body = res.json();
    // If mutation returns just a string id
    if (body && body.data && body.data.createUserPayment) {
      const val = body.data.createUserPayment;
      // If API returns id as string in data.createUserPayment
      if (typeof val === 'string') {
        return { res, id: val };
      }
      // If API returns object
      if (val && val.id) {
        return { res, id: val.id };
      }
    }
  } catch (e) {
    // ignore parse errors
  }
  return { res, id: null };
}

function performApproveOrRejectPayment(id, newState, commentary) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const variables = { id, newState, commentary };
  const res = graphqlRequest(GRAPHQL_ENDPOINT, APPROVE_REJECT_MUTATION, variables, headers);
  check(res, { 'approve/reject payment 200': (r) => r.status === 200 });
  return res;
}

function performGetAllPayments() {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_ENDPOINT, GET_ALL_PAYMENTS_QUERY, {}, headers);
  check(res, { 'get all payments 200': (r) => r.status === 200 });
  return res;
}

function performGetUserPayments(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_ENDPOINT, GET_USER_PAYMENTS_QUERY, { accessToken }, headers);
  check(res, { 'get user payments 200': (r) => r.status === 200 });
  return res;
}

function performGetHistory(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_ENDPOINT, GET_HISTORY_QUERY, { accessToken }, headers);
  check(res, { 'get history 200': (r) => r.status === 200 });
  return res;
}

function performGetUserLoans(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_LOAN_ENDPOINT, GET_USER_LOANS_QUERY, { accessToken }, headers);
  check(res, { 'get user loans 200': (r) => r.status === 200 });
  return res;
}

function performGetFinesById(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_FINE_ENDPOINT, GET_FINES_BY_ID_QUERY, { accessToken }, headers);
  check(res, { 'get fines 200': (r) => r.status === 200 });
  return res;
}

function performCreateFine(affiliateKey) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const variables = { affiliateKey, amount: 10.0, motive: 'k6 fine test' };
  const res = graphqlRequest(GRAPHQL_FINE_ENDPOINT, CREATE_FINE_MUTATION, variables, headers);
  check(res, { 'create fine 200': (r) => r.status === 200 });
  return res;
}

function performGetPendingQuotas(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_QUOTA_ENDPOINT, GET_PENDING_QUOTAS_QUERY, { accessToken }, headers);
  check(res, { 'get pending quotas 200': (r) => r.status === 200 });
  return res;
}

function performGetMonthlyAffiliateQuota(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_QUOTA_ENDPOINT, GET_MONTHLY_AFFILIATE_QUOTA_QUERY, { accessToken }, headers);
  check(res, { 'get monthly affiliate quota 200': (r) => r.status === 200 });
  return res;
}

function performGetQuotasPrestamoPendientes(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_QUOTA_ENDPOINT, GET_QUOTAS_PRESTAMO_PENDIENTES_QUERY, { accessToken }, headers);
  check(res, { 'get quotas prestamo pendientes 200': (r) => r.status === 200 });
  return res;
}

function performGetPendingLoansQuotas(accessToken) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_QUOTA_ENDPOINT, GET_PENDING_LOANS_QUOTAS_QUERY, { accessToken }, headers);
  check(res, { 'get pending loans quotas 200': (r) => r.status === 200 });
  return res;
}

function performGetAllMembers() {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const res = graphqlRequest(GRAPHQL_ENDPOINT, GET_ALL_MEMBERS_QUERY, {}, headers);
  check(res, { 'get all members 200': (r) => r.status === 200 });
  return res;
}

function performEditFine(fineKey, newAmount, newMotive) {
  const headers = { headers: { 'Content-Type': 'application/json' } };
  const variables = { fineKey, newAmount, newMotive, newStatus: null };
  const res = graphqlRequest(GRAPHQL_FINE_ENDPOINT, EDIT_FINE_MUTATION, variables, headers);
  check(res, { 'edit fine 200': (r) => r.status === 200 });
  return res;
}

export function setup() {
  const token = signupAndGetToken();
  if (!token) {
    throw new Error('Signup did not return access_token; aborting');
  }
  return { accessToken: token };
}

export function smokeScenario(data) {
  const token = data.accessToken;
  const created = performCreatePayment(token);
  // if id returned, attempt to approve it
  if (created && created.id) {
    // attempt to approve the payment (toggle to ACCEPTED)
    performApproveOrRejectPayment(created.id, 'ACCEPTED', 'approved by k6 smoke');
  }
  sleep(0.5);
  performGetAllPayments();
  sleep(0.5);
  performGetUserPayments(token);
  sleep(0.5);
  performGetHistory(token);
  sleep(0.5);
  performGetUserLoans(token);
  sleep(0.5);
  performGetFinesById(token);
  sleep(0.5);
  performGetPendingQuotas(token);
  sleep(0.5);
  performGetMonthlyAffiliateQuota(token);
  sleep(0.5);
  performGetQuotasPrestamoPendientes(token);
  sleep(0.5);
  performGetPendingLoansQuotas(token);
  sleep(0.5);
  performGetAllMembers();
}

export function fullScenario(data) {
  const token = data.accessToken;
  // each VU will perform create + get in loop
  const created = performCreatePayment(token);
  if (created && created.id) {
    performApproveOrRejectPayment(created.id, 'ACCEPTED', 'approved by k6 full');
  }
  sleep(0.5);
  performGetAllPayments();
  sleep(1);
}
