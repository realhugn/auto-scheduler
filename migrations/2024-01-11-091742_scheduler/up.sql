-- Your SQL goes here
CREATE TABLE IF NOT EXISTS Employees (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    phone_number TEXT ,
    department TEXT,
    role TEXT NOT NULL,
    availability JSON
);

CREATE TABLE IF NOT EXISTS Shifts (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    start_time INT NOT NULL,
    end_time INT NOT NULL,
    duration INT,
    minium_attendences INT DEFAULT 0
);

CREATE TABLE IF NOT EXISTS Schedules (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    employee_id INT NOT NULL,
    data DATE NOT NULL,
    shift_id INT NOT NULL,
    note TEXT,
    FOREIGN KEY(employee_id) REFERENCES Employees(id),
    FOREIGN KEY(shift_id) REFERENCES Shifts(id)
);

CREATE TABLE IF NOT EXISTS Shift_Changes(
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    scheduler_id INT UNIQUE NOT NULL,
    reason TEXT,
    status TEXT DEFAULT 'pending',
    FOREIGN KEY(scheduler_id) REFERENCES Schedules(id)
);

INSERT INTO Shifts(name, start_time, end_time, duration, minium_attendences) VALUES ('H', 2, 12, 10, 1);
INSERT INTO Shifts(name, start_time, end_time, duration, minium_attendences) VALUES ('D', 16, 24, 8, 1);
INSERT INTO Shifts(name, start_time, end_time, duration, minium_attendences) VALUES ('C', 8, 16, 8, 1);
INSERT INTO Shifts(name, start_time, end_time, duration, minium_attendences) VALUES ('S', 0, 8, 8, 1);