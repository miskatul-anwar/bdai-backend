#!/usr/bin/env python3
"""
Seed initial held and upcoming events into Supabase site_settings table.
"""

import os
import json
import urllib.request
from urllib.error import HTTPError

SUPABASE_URL = "https://qcqwzeaoukkhfvhrbygf.supabase.co"
SUPABASE_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InFjcXd6ZWFvdWtraGZ2aHJieWdmIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc5MDI3MjIwMiwiZXhwIjoyMTA1ODQ4MjAyfQ.47HNRUWYhZ20-OAQ00K4wNxibpUW9azY3nmBtgn4o0k"

EVENTS_DATA = [
    {
        "id": "event_workshop_debasish",
        "title": "Professor Dr. Debasish Ghose from Kristiania University College, Norway visited our lab for collaboration purpose. He delivers an intensive quality paper writing workshop.",
        "date": "29th July 2026",
        "status": "held",
        "category": "Workshop",
        "location": "BDAI Lab & SPMT Office, Department of CSE, University of Chittagong",
        "description": "Professor Dr. Debasish Ghose from Kristiania University College, Norway visited our lab for research collaboration and delivered an intensive quality paper writing workshop for researchers and faculty members.",
        "banner": "/events/workshop_banner.jpeg",
        "gallery": [
            {"src": "/events/workshop_1.jpeg", "alt": "Workshop participants gathered with Prof. Dr. Debasish Ghose"},
            {"src": "/events/workshop_2.jpeg", "alt": "Collaborators and researchers in the department hallway"},
            {"src": "/events/workshop_3.jpeg", "alt": "Prof. Dr. Debasish Ghose, Prof. Dr. Rudra Pratap Deb Nath, and Dr. Abu Nowshed Chy at SPMT office"},
            {"src": "/events/workshop_4.jpeg", "alt": "Faculty and visiting professor outside SPMT office"},
            {"src": "/events/workshop_5.jpeg", "alt": "Collaboration meeting at SPMT office"},
            {"src": "/events/workshop_6.jpeg", "alt": "Research discussion at SPMT office"},
            {"src": "/events/workshop_7.jpeg", "alt": "Faculty collaboration outside BIKE Lab SPMT office"},
            {"src": "/events/workshop_8.jpeg", "alt": "Group photo in the BDAI lab"}
        ],
        "order": 1
    },
    {
        "id": "event_seminar_rag_bi",
        "title": "RAG-Driven Business Intelligence Platform Integration: Enterprise Data for Real-Time Insight, Predictive, and Prescriptive Decision Analytics",
        "date": "2.00PM · 19th May 2026",
        "status": "held",
        "category": "Seminar",
        "location": "Department of Computer Science and Engineering, University of Chittagong",
        "description": "Seminar on enterprise integration of retrieval-augmented generation and semantic knowledge graphs for real-time analytics and predictive decision systems.",
        "banner": "/events/seminar2.jpg",
        "gallery": [
            {"src": "/events/seminar2_1.jpeg", "alt": "RAG-Driven BI seminar gallery image 1"},
            {"src": "/events/seminar2_2.jpeg", "alt": "RAG-Driven BI seminar gallery image 2"},
            {"src": "/events/seminar2_3.jpeg", "alt": "RAG-Driven BI seminar gallery image 3"},
            {"src": "/events/seminar2_4.jpeg", "alt": "RAG-Driven BI seminar gallery image 4"},
            {"src": "/events/seminar2_5.jpeg", "alt": "RAG-Driven BI seminar gallery image 5"},
            {"src": "/events/seminar2_6.jpeg", "alt": "RAG-Driven BI seminar gallery image 6"}
        ],
        "order": 2
    },
    {
        "id": "event_phd_cyberbullying",
        "title": "Identificatin of the Digital Footprints of Cyberbullying and the personality traits of the perpretators to protect the malicious activity",
        "date": "2.00PM · 14th May 2026",
        "status": "held",
        "category": "PhD Seminar",
        "location": "Department of Computer Science and Engineering, University of Chittagong",
        "description": "PhD Open Seminar on machine learning models and digital footprint analysis for cyberbullying detection and perpetrator personality classification in Bengali social text.",
        "banner": "/events/1.png",
        "gallery": [
            {"src": "/events/phd1.png", "alt": "Event gallery image 1"},
            {"src": "/events/phd2.png", "alt": "Event gallery image 2"},
            {"src": "/events/phd3.png", "alt": "Event gallery image 3"},
            {"src": "/events/phd4.png", "alt": "Event gallery image 4"},
            {"src": "/events/phd5.png", "alt": "Event gallery image 5"},
            {"src": "/events/phd6.png", "alt": "Event gallery image 6"}
        ],
        "order": 3
    }
]

def seed_events():
    url = f"{SUPABASE_URL}/rest/v1/site_settings"
    headers = {
        "apikey": SUPABASE_KEY,
        "Authorization": f"Bearer {SUPABASE_KEY}",
        "Content-Type": "application/json",
        "Prefer": "resolution=merge-duplicates"
    }

    payload = {
        "id": "events",
        "data": EVENTS_DATA
    }

    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers=headers, method="POST")

    try:
        with urllib.request.urlopen(req) as response:
            print(f"✅ Successfully seeded events into Supabase site_settings (HTTP {response.getcode()})")
    except HTTPError as e:
        print(f"❌ Failed to seed events: HTTP {e.code} - {e.read().decode('utf-8')}")

if __name__ == "__main__":
    seed_events()
