import os
import sys
import unittest
from io import BytesIO

# Ensure backend package is importable
BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, BASE_DIR)

from app import app as flask_app

class SteelPipeAPITest(unittest.TestCase):
    def setUp(self):
        self.client = flask_app.test_client()
        self.ctx = flask_app.app_context()
        self.ctx.push()
        # Clean up database to ensure clean state for tests
        db_path = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'pipes.db'))
        if os.path.exists(db_path):
            os.remove(db_path)

    def tearDown(self):
        self.ctx.pop()

    def test_crud_and_io(self):
        # 1. GET empty
        rv = self.client.get('/pipes')
        self.assertEqual(rv.status_code, 200)
        data = rv.get_json()
        self.assertIn('pipes', data)

        # 2. POST new pipe
        payload = {"diameter": 12.0, "thickness": 1.2, "length": 6.0, "material": "Carbon Steel", "quantity": 5}
        rv = self.client.post('/pipes', json=payload)
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

        # 9. Export (CSV)
        # Add two records then export
        self.client.post('/pipes', json={"diameter": 10.0, "thickness": 1.0, "length": 5.0, "material": "Carbon Steel", "quantity": 2})
        self.client.post('/pipes', json={"diameter": 12.0, "thickness": 1.0, "length": 6.0, "material": "Stainless Steel", "quantity": 3})
        rv = self.client.get('/pipes/export?format=csv')
        self.assertEqual(rv.status_code, 200)
        self.assertIn(b'id,diameter,thickness,length,material,quantity', rv.data)

        # 10. Import CSV
        csv_content = "diameter,thickness,length,material,quantity\n11,1.0,4,Carbon Steel,3\n13,1.2,5,Alloy Steel,4\n"
        data = {'file': (BytesIO(csv_content.encode('utf-8')), 'pipes.csv')}
        rv = self.client.post('/pipes/import', data=data, content_type='multipart/form-data')
        self.assertEqual(rv.status_code, 200)
        res = rv.get_json()
        self.assertIn('imported', res)
        self.assertEqual(res['imported'], 2)

if __name__ == '__main__':
    unittest.main()
