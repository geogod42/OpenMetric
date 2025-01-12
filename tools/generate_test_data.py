#!/usr/bin/env python3

"""
generate_test_data.py

Generates extremely realistic .evnt and .ret files for testing.

Usage:
  python generate_test_data.py

Outputs:
  - data/MyCompany.evnt
  - data/MyCompany.ret

Place in: /tools/generate_test_data.py

Author: Jude Hoogterp
"""

import os
import json
import random
from datetime import datetime, timedelta, timezone

# ---------------------------
# CONFIGURABLE PARAMETERS
# ---------------------------

# Where to save the files
DATA_OUTPUT_DIR = "/Users/jude/Desktop/OpenMetric/OpenMetric/data"

# The base filename (e.g. "MyCompany"); will produce "MyCompany.evnt" and "MyCompany.ret"
COMPANY_NAME = "TESTCompany2"

# Date range for generating data (YYYY, M, D)
START_DATE = datetime(2016, 1, 1)
END_DATE   = datetime(2024, 12, 31)

# The approximate number of paying customers per month
AVG_CUSTOMERS_PER_MONTH = 30

# Probability of an expense event per payment event
EXPENSE_PROBABILITY = 0.5

# Probability of a churn/cancellation event (per month, per active customer)
CANCELLATION_PROB = 0.03

# Range for payment amounts (in USD)
PAYMENT_MIN = 1000.0
PAYMENT_MAX = 10000.0

# Range for expense amounts (in USD)
EXPENSE_MIN = 500.0
EXPENSE_MAX = 10000.0

# Number of "active" weeks to track in retention
RETENTION_WEEKS = 4

# Set a random seed for reproducibility (optional)
RANDOM_SEED = 42

# ---------------------------
# HELPER FUNCTIONS
# ---------------------------

def daterange(start_date, end_date):
    """
    Generator that yields each date from start_date to end_date (inclusive).
    """
    current = start_date
    while current <= end_date:
        yield current
        current += timedelta(days=1)

def month_key(dt: datetime) -> str:
    """
    Returns a string "YYYY-MM" for a given datetime.
    """
    return f"{dt.year:04d}-{dt.month:02d}"

def random_payment_amount():
    """Generate a random payment amount."""
    return round(random.uniform(PAYMENT_MIN, PAYMENT_MAX), 2)

def random_expense_amount():
    """Generate a random expense amount."""
    return round(random.uniform(EXPENSE_MIN, EXPENSE_MAX), 2)

def random_day_in_month(some_month: datetime) -> datetime:
    """
    Given a datetime corresponding to the 1st day of a month,
    pick a random day within that same month, including a random time and timezone.
    """
    year = some_month.year
    month = some_month.month
    if month == 12:
        next_month = some_month.replace(year=year + 1, month=1, day=1)
    else:
        next_month = some_month.replace(month=month + 1, day=1)

    days_in_month = (next_month - some_month).days
    random_day = random.randint(1, days_in_month)
    random_time = timedelta(hours=random.randint(0, 23), minutes=random.randint(0, 59), seconds=random.randint(0, 59))
    return some_month.replace(day=random_day, tzinfo=timezone.utc) + random_time

def generate_evnt_data():
    """
    Generate a list of event dicts with structure:
    [
      {
        "event_type": "payment" | "expense" | "cancellation",
        "customer_id": int,
        "amount": float,
        "description": str,
        "timestamp": str (RFC 3339),
      }, ...
    ]
    """
    events = []

    active_customers = set()
    next_customer_id = 1

    current_month = START_DATE.replace(day=1)
    while current_month <= END_DATE:
        new_customers_count = int(random.gauss(AVG_CUSTOMERS_PER_MONTH, 0.2 * AVG_CUSTOMERS_PER_MONTH))
        new_customers_count = max(new_customers_count, 0)

        newly_acquired_ids = []
        for _ in range(new_customers_count):
            newly_acquired_ids.append(next_customer_id)
            active_customers.add(next_customer_id)
            next_customer_id += 1

        for cust_id in list(active_customers):
            event = {
                "event_type": "payment",
                "customer_id": cust_id,
                "amount": random_payment_amount(),
                "description": "Monthly subscription payment",
                "timestamp": random_day_in_month(current_month).isoformat()
            }
            events.append(event)

            if random.random() < EXPENSE_PROBABILITY:
                event = {
                    "event_type": "expense",
                    "customer_id": None,
                    "amount": random_expense_amount(),
                    "description": "Various overhead costs",
                    "timestamp": random_day_in_month(current_month).isoformat()
                }
                events.append(event)

        for cust_id in list(active_customers):
            if random.random() < CANCELLATION_PROB:
                event = {
                    "event_type": "cancellation",
                    "customer_id": cust_id,
                    "amount": None,
                    "description": "Customer churn",
                    "timestamp": random_day_in_month(current_month).isoformat()
                }
                events.append(event)
                active_customers.remove(cust_id)

        year = current_month.year
        month = current_month.month + 1
        if month > 12:
            month = 1
            year += 1
        current_month = current_month.replace(year=year, month=month)

    return events

def generate_ret_data(events):
    """
    Generate retention data based on events.
    """
    customer_first_month = {}

    for e in events:
        if e["event_type"] == "payment" and e["customer_id"] is not None:
            dt = datetime.fromisoformat(e["timestamp"])
            mkey = month_key(dt)
            cust_id = e["customer_id"]
            if cust_id not in customer_first_month:
                customer_first_month[cust_id] = mkey

    ret_map = {}
    for cust_id, mkey in customer_first_month.items():
        if mkey not in ret_map:
            ret_map[mkey] = {
                "acquired": 0,
                "active": [0] * RETENTION_WEEKS
            }
        ret_map[mkey]["acquired"] += 1

    for mkey, data in ret_map.items():
        current_active = data["acquired"]
        for i in range(RETENTION_WEEKS):
            decay_rate = 0.85 + random.uniform(-0.05, 0.05)
            if i == 0:
                data["active"][i] = current_active
            else:
                current_active = int(current_active * decay_rate)
                data["active"][i] = current_active

    return ret_map

# ---------------------------
# MAIN
# ---------------------------

def main():
    random.seed(RANDOM_SEED)

    evnt_data = generate_evnt_data()
    ret_data = generate_ret_data(evnt_data)

    os.makedirs(DATA_OUTPUT_DIR, exist_ok=True)

    evnt_file_path = os.path.join(DATA_OUTPUT_DIR, f"{COMPANY_NAME}.evnt")
    ret_file_path  = os.path.join(DATA_OUTPUT_DIR, f"{COMPANY_NAME}.ret")

    with open(evnt_file_path, 'w') as f:
        json.dump(evnt_data, f, indent=2)
    print(f"Saved {evnt_file_path}")

    with open(ret_file_path, 'w') as f:
        json.dump(ret_data, f, indent=2)
    print(f"Saved {ret_file_path}")

if __name__ == "__main__":
    main()
