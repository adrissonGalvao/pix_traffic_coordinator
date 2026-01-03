# Pix_traffic_coordinator


> **Context:** Instant Payment Gateway (PIX/LBTR).
> **Stack:** Rust + Redis Cluster + Lua Scripting.
> **Model:** Hierarchical Token Bucket with Penalty and Debt Clamping.

---

## 1. Overview and Purpose

`pix_traffic_coordinator` acts as the central traffic controller for Pix transactions. It implements a defense-in-depth model that protects both the internal infrastructure and the PSP's regulatory SLAs.

The architecture uses a **Dual Check (Global + User)** model with an **Optimistic Debit and Error Feedback** mechanism. The system allows entry based on a positive balance but applies severe penalties in case of settlement failure, creating a temporal debt. To prevent infinite penalties (Eternal Ban), the system imposes a configurable "Debt Floor."

---

## 2. System Parameters

The algorithm's behavior is governed by configurable parameters that define physical limits and business rules.

### 2.1. Variable Definitions

| Parameter              | Symbol        | Description                                                                          |
| :--------------------- | :------------ | :----------------------------------------------------------------------------------- |
| **Bucket Capacity**    | $B$           | Maximum number of accumulated tokens (*Burst Size*).                                 |
| **Refill Rate**        | $R_{sec}$     | Replenishment rate in Tokens per Second (calculated from tokens/minute).             |
| **Base Cost**          | $C_{base}$    | Standard cost to initiate a transaction (usually 1).                                 |
| **Penalty Cost**       | $C_{penalty}$ | Additional tokens consumed in case of an error.                                      |
| **Maximum Debt Limit** | $D_{max}$     | The negative "floor" of the balance. The balance will never be less than $-D_{max}$. |


### 2.2. Configuration Source
The above parameters are loaded at application startup from the `ratelimit_policy.yml` file. This file allows fine-tuning of *burst* and *penalty* behavior without recompiling the Rust code.

```yaml
policy:
  # Bucket Capacity (Burst Size) - B
  # Maximum number of accumulated tokens.
  max_tokens: 100

  # Refill Rate - R_min
  # How many tokens are generated per MINUTE.
  refill_per_minute: 600

  # Penalty Cost - C_penalty
  # Additional tokens consumed in case of an error.
  error_penalty_tokens: 3

  # Maximum Debt Limit - D_max
  # The balance will never be less than the negative of this number.
  # Ex: if 20, the minimum balance is -20.
  max_negative_debt: 20
```
---

## 3. Mathematical Foundation: Clamping and Penalties

The system uses two independent buckets (Global and User) that must be satisfied simultaneously. The algorithm includes a clamping function to ensure that the debt does not exceed the parameterized limit.

### 3.1. State Variables (Redis)

| Variable | Definition |
| : | : |
| **$t_{now}$** | Current timestamp (Redis Server Time). |
| **$t_{last}$** | Timestamp of the last update. |
| **$T_{curr}$** | Current balance (Can be negative). |

### 3.2. Refill Logic (With Debt Support)
The replenishment pays off the debt before accumulating a positive balance. The logic is applied identically for the Global Bucket and the User Bucket.

$$
\begin{aligned}
1.\ \Delta t &= t_{now} - t_{last} \\
2.\ \text{generated} &= \Delta t \times R_{sec} \\
3.\ T_{new} &= \min(B, T_{curr} + \text{generated})
\end{aligned}
$$

### 3.3. Optimistic Consumption Logic (Entry)
To start a transaction, **both** buckets must have a strictly positive balance sufficient for the base cost ($C_{base} = 1$).

$$
\text{Admit} \iff (T_{user\_new} \ge C_{base}) \land (T_{global\_new} \ge C_{base})
$$

*   **If accepted:** Decrements $C_{base}$ from both and updates $t_{last}$.
*   **If rejected:** Returns `429 Too Many Requests` and calculates the `Retry-After`.

### 3.4. Penalty Logic (Post-Error)
If the transaction fails, we apply the penalty respecting the lower limit (Clamping).

$$T_{penalized} = T_{curr} - C_{penalty}$$
$$T_{final} = \max(-D_{max}, T_{penalized})$$

> **Practical Example:**
> *   Config: $D_{max} = 20$ (Floor is -20).
> *   Current State: User has $-18$ tokens.
> *   Action: A new error occurs (Cost 3).
> *   Calculation: $-18 - 3 = -21$.
> *   Clamping: The balance is updated to **-20** (not -21).

---

## 4. Implementation Architecture

### 4.1. Request Lifecycle

1.  **Initialization:** Rust loads the configuration parameters.
2.  **Request Inbound:** Rust receives the Pix intent.
3.  **Lua Script (Check & Reserve):**
    *   Executes the Refill logic for `rl:user:{id}` and `rl:psp:global`.
    *   Checks if both have a balance $\ge 1$.
    *   If **OK**: Consumes 1, returns `Allowed`.
    *   If **NOK**: Returns `429` and the estimated time to get out of the negative (TTL).
4.  **External Execution:** Calls the Central Bank / Core Banking API.
5.  **Error Compensation (Asynchronous):**
    *   If there is a failure, Rust triggers a Lua Penalty command.
    *   The script subtracts tokens and applies `math.max(-D_max, new_balance)`.

### 4.2. Debt Handling

When $T_{curr} < 0$, the user is in "Cool-down". The time to unblock ($T_{wait}$) is calculated by:

$$T_{wait} (seconds) = \frac{|T_{curr}| + C_{base}}{R_{sec}}$$

The Gateway must return this value in the `Retry-After` header of the 429 response.

---

## 5. Engineering Guidelines

*   **Parameter Validation:** The system must fail on boot if $D_{max}$ is not consistent or if the refill rate is zero.
*   **Floating Point Precision:** As we are converting per-minute rates to per-second, $R_{sec}$ will be fractional. Redis must store the balance as a *Float* and not an *Int*, otherwise, low refill rates will be rounded to zero.
*   **Fail-Safe:** The debt limit ($D_{max}$) is a safety protection. It ensures that no retry loop bug or brute-force attack causes a denial of service to the user for an indefinite time.
*   **Observability:** Distinct metrics for `UserLimitHit` vs `GlobalLimitHit`.
