import os
import sys
import unittest
from app import app as flask_app

class FrontendFrontendTest(unittest.TestCase):
    def setUp(self):
        self.client = flask_app.test_client()
        self.ctx = flask_app.app_context()
        self.ctx.push()

    def tearDown(self):
        self.ctx.pop()

    def test_index_serves_frontend(self):
        rv = self.client.get('/')
        self.assertEqual(rv.status_code, 200)
        self.assertIn(b'Steel Pipe Inventory', rv.data)

if __name__ == '__main__':
    unittest.main()
