import os
import sys
import unittest
from io import BytesIO

BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, BASE_DIR)

from app import app as flask_app, init_db

class SteelPipeAPITest(unittest.TestCase):
    def setUp(self):
        self.client = flask_app.test_client()
        self.ctx = flask_app.app_context()
        self.ctx.push()
        db_path = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'pipes.db'))
        if os.path.exists(db_path):
            os.remove(db_path)
        open(db_path, 'a').close()
        init_db()

    def tearDown(self):
        self.ctx.pop()

    def test_crud_and_io(self):
        # 1. GET empty
        rv = self.client.get('/pipes')
        self.assertEqual(rv.status_code, 200)
        data = rv.get_json()
        self.assertIn('pipes', data)

        # 2. POST new pipe (use entry endpoint for full pipe data)
        payload = {"pipe_id": "PIPE-001", "diameter": 12.0, "thickness": 1.2, "length": 6.0, "material": "Carbon Steel", "quantity": 5, "operator": "test"}
        rv = self.client.post('/pipes/entry', json=payload)
        self.assertEqual(rv.status_code, 201)
        self.assertEqual(rv.get_json().get('status'), 'created')

        # 3. GET list -> total 1
        rv = self.client.get('/pipes')
        data = rv.get_json()
        self.assertEqual(data.get('total'), 1)

        # 4. GET /pipes/1
        rv = self.client.get('/pipes/1')
        d = rv.get_json()
        self.assertEqual(d['diameter'], 12.0)

        # 5. PUT update
        updated = {"diameter": 14.0, "thickness": 1.5, "length": 6.5, "material": "Carbon Steel", "quantity": 7}
        rv = self.client.put('/pipes/1', json=updated)
        self.assertEqual(rv.status_code, 200)
        self.assertEqual(rv.get_json().get('status'), 'updated')

        # 6. GET updated
        rv = self.client.get('/pipes/1')
        d = rv.get_json()
        self.assertEqual(d['diameter'], 14.0)

        # 7. DELETE
        rv = self.client.delete('/pipes/1')
        self.assertEqual(rv.status_code, 200)
        self.assertEqual(rv.get_json().get('status'), 'deleted')

        # 8. GET empty again
        rv = self.client.get('/pipes')
        data = rv.get_json()
        self.assertEqual(data.get('total'), 0)

        # 9. Test statistics endpoint
        rv = self.client.get('/statistics')
        self.assertEqual(rv.status_code, 200)
        stats = rv.get_json()
        self.assertIn('total_types', stats)

        # 10. Test entry and exit
        self.client.post('/pipes/entry', json={"pipe_id": "PIPE-002", "diameter": 10.0, "thickness": 1.0, "length": 5.0, "material": "Steel", "quantity": 10, "operator": "test"})
        rv = self.client.post('/pipes/exit', json={"pipe_id": "PIPE-002", "quantity": 3, "operator": "test"})
        self.assertEqual(rv.status_code, 200)

        # 11. Test records endpoint
        rv = self.client.get('/records')
        self.assertEqual(rv.status_code, 200)
        records = rv.get_json()
        self.assertTrue(len(records) > 0)

if __name__ == '__main__':
    unittest.main()
